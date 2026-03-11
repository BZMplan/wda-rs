mod db;
mod router;
mod structure;
mod utils;

use axum::Router;
use axum::routing::{get, post};
use chrono::{DateTime, Local};
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::Level;

use crate::utils::load_config;
use router::{get_data_with_path, get_data_with_query, route_not_found, upload_data};
use sqlx::postgres::PgPoolOptions;
use tower_http::trace::{DefaultMakeSpan, DefaultOnFailure, TraceLayer};
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct RotatingLatestLogger {
    inner: Arc<Mutex<LogState>>,
}

struct LogState {
    dir: PathBuf,
    current_date: String,
    file: Option<File>,
}

struct RotatingWriter {
    inner: Arc<Mutex<LogState>>,
}

impl RotatingLatestLogger {
    fn new<P: AsRef<Path>>(dir: P) -> io::Result<Self> {
        fs::create_dir_all(&dir)?;
        let dir = dir.as_ref().to_path_buf();
        let latest_path = dir.join("latest.log");
        let current_date = if latest_path.exists() {
            let modified = fs::metadata(&latest_path)?.modified()?;
            format_date(DateTime::<Local>::from(modified))
        } else {
            today_string()
        };
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&latest_path)?;

        Ok(Self {
            inner: Arc::new(Mutex::new(LogState {
                dir,
                current_date,
                file: Some(file),
            })),
        })
    }
}

impl LogState {
    fn rotate_if_needed(&mut self) -> io::Result<()> {
        let today = today_string();
        if today == self.current_date {
            return Ok(());
        }

        if let Some(mut file) = self.file.take() {
            file.flush()?;
        }

        let latest_path = self.dir.join("latest.log");
        let archived_path = self.dir.join(format!("{}.log", self.current_date));

        if latest_path.exists() {
            if archived_path.exists() {
                fs::remove_file(&archived_path)?;
            }
            fs::rename(&latest_path, &archived_path)?;
        }

        let new_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&latest_path)?;
        self.file = Some(new_file);
        self.current_date = today;
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for RotatingLatestLogger {
    type Writer = RotatingWriter;

    fn make_writer(&'a self) -> Self::Writer {
        RotatingWriter {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Write for RotatingWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut state = self
            .inner
            .lock()
            .map_err(|_| io::Error::other("failed to lock logger state"))?;
        state.rotate_if_needed()?;
        state
            .file
            .as_mut()
            .ok_or_else(|| io::Error::other("log file unavailable"))?
            .write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut state = self
            .inner
            .lock()
            .map_err(|_| io::Error::other("failed to lock logger state"))?;
        state
            .file
            .as_mut()
            .ok_or_else(|| io::Error::other("log file unavailable"))?
            .flush()
    }
}

fn today_string() -> String {
    format_date(Local::now())
}

fn format_date(datetime: DateTime<Local>) -> String {
    datetime.format("%Y-%m-%d").to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let file_writer = RotatingLatestLogger::new("logs")?;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_ansi(false)
                .with_target(false)
                .with_line_number(true),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(file_writer),
        )
        .init();

    let config = load_config()?;
    let database_url = config.database.connection_url();
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await?;

    // build our application with a single route
    let app = Router::new()
        .route("/upload", post(upload_data))
        .route("/get", get(get_data_with_query))
        .route("/get/{station_id}", get(get_data_with_path))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(
                    |res: &axum::http::Response<_>, latency: Duration, _span: &tracing::Span| {
                        tracing::info!(
                            status = res.status().as_u16(),
                            latency_ms = latency.as_secs_f64() * 1000.0,
                            "request completed"
                        );
                    },
                )
                .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
        )
        .with_state(pool)
        .fallback(route_not_found);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    tracing::info!("Server started on 0.0.0.0:3000");

    axum::serve(listener, app).await?;
    Ok(())
}
