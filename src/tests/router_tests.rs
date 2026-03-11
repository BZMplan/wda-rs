use axum::{Json, body::to_bytes, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::Value;

use crate::router::{AppError, get_data, route_not_found, upload_data, upload_data_batch};
use crate::tests::common::{lazy_pool, sample_upload};

#[tokio::test]
async fn app_error_into_response_contains_expected_json() {
    let response = AppError::bad_request("invalid input").into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should be readable");
    let value: Value = serde_json::from_slice(&body).expect("body should be valid json");
    assert_eq!(value["code"], 400);
    assert_eq!(value["message"], "invalid input");
}

#[tokio::test]
async fn route_not_found_returns_standard_payload() {
    let response = route_not_found().await.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should be readable");
    let value: Value = serde_json::from_slice(&body).expect("body should be valid json");
    assert_eq!(value["code"], 404);
    assert_eq!(value["message"], "route not found");
}

#[tokio::test]
async fn upload_data_rejects_non_positive_station_id() {
    let pool = lazy_pool();
    let result = upload_data(State(pool), Json(sample_upload(0, None))).await;
    let err = result.expect_err("invalid station_id should fail fast");
    let response = err.into_response();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn upload_data_batch_rejects_empty_payload() {
    let pool = lazy_pool();
    let result = upload_data_batch(State(pool), Json(Vec::new())).await;
    let err = result.expect_err("empty batch should fail fast");
    let response = err.into_response();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_data_rejects_non_positive_station_id() {
    let pool = lazy_pool();
    let result = get_data(&pool, 0).await;
    let err = result.expect_err("invalid station_id should fail fast");
    let response = err.into_response();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
