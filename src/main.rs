mod db;
mod router;
mod structure;
mod utils;

use axum::Router;
use axum::routing::{get, post};
use tracing::info;

use crate::utils::load_config;
use once_cell::sync::Lazy;
use router::{get_data_with_path, get_data_with_query, route_not_found, upload_data};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// 定义全局数据库连接池
pub static DB_POOL: Lazy<PgPool> = Lazy::new(|| {
    let config = load_config().unwrap();
    let url = format!(
        "postgre://{}:{}@{}:{}/{}",
        config.database.user,
        config.database.password,
        config.database.host,
        config.database.port,
        config.database.db_name
    );
    PgPoolOptions::new()
        .max_connections(20)
        .connect_lazy(&url)
        .expect("db pool create failed")
});

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

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
        .fallback(route_not_found);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::info!("Server started on 0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}
