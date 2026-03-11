mod db;
mod router;
mod structure;
mod utils;

use axum::Router;
use axum::routing::{get, post};
use std::error::Error;
use tracing::info;

use crate::utils::load_config;
use router::{get_data_with_path, get_data_with_query, route_not_found, upload_data};
use sqlx::postgres::PgPoolOptions;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
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
        .layer(TraceLayer::new_for_http().on_request(
            |req: &axum::http::Request<_>, _span: &tracing::Span| {
                info!("{} {}", req.method(), req.uri());
            },
        ))
        .with_state(pool)
        .fallback(route_not_found);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    tracing::info!("Server started on 0.0.0.0:3000");

    axum::serve(listener, app).await?;
    Ok(())
}
