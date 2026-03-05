mod router;
mod structure;
mod db;
mod utils;

use axum::Router;
use axum::routing::{get, post};

use router::get_data;
use router::get_data_query;
use router::upload_data;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/upload", post(upload_data))
        .route("/get/{param_type}/{param}", get(get_data))
        .route("/get", get(get_data_query));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
