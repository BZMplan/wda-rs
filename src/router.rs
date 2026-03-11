use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgPool;
use tracing::{error, instrument};

use crate::db::{insert_weather_data, insert_weather_data_batch, query_latest_weather_data};
use crate::structure::ElemUpload;

#[derive(Deserialize)]
pub struct GetParams {
    station_id: i32,
}

#[derive(Debug)]
pub struct AppError {
    status: StatusCode,
    message: String,
}

impl AppError {
    fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub(crate) fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        error!(error = %err, "database operation failed");
        Self::internal("database operation failed")
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "code": self.status.as_u16(),
            "message": self.message
        }));
        (self.status, body).into_response()
    }
}

type ApiResult = Result<(StatusCode, Json<Value>), AppError>;

pub async fn get_data_with_path(
    State(pool): State<PgPool>,
    Path(params): Path<GetParams>,
) -> ApiResult {
    get_data(&pool, params.station_id).await
}

pub async fn get_data_with_query(
    State(pool): State<PgPool>,
    Query(params): Query<GetParams>,
) -> ApiResult {
    get_data(&pool, params.station_id).await
}

#[instrument(skip(pool), fields(station_id = station_id))]
pub(crate) async fn get_data(pool: &PgPool, station_id: i32) -> ApiResult {
    if station_id <= 0 {
        return Err(AppError::bad_request("station_id must be positive"));
    }

    let data = query_latest_weather_data(pool, station_id)
        .await?
        .ok_or_else(|| AppError::not_found("station data not found"))?;

    Ok((StatusCode::OK, Json(json!(data))))
}

#[instrument(skip(pool, payload), fields(station_id = payload.station.station_id))]
pub async fn upload_data(
    State(pool): State<PgPool>,
    Json(mut payload): Json<ElemUpload>,
) -> ApiResult {
    if payload.station.station_id <= 0 {
        return Err(AppError::bad_request("station_id must be positive"));
    }

    payload.build();
    let response = json!({
        "timestamp": payload.timestamp,
        "station": payload.station,
        "weather": payload.weather
    });

    insert_weather_data(&pool, &payload).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool, payloads), fields(size = payloads.len()))]
pub async fn upload_data_batch(
    State(pool): State<PgPool>,
    Json(mut payloads): Json<Vec<ElemUpload>>,
) -> ApiResult {
    if payloads.is_empty() {
        return Err(AppError::bad_request("payload must not be empty"));
    }

    for payload in &mut payloads {
        if payload.station.station_id <= 0 {
            return Err(AppError::bad_request("station_id must be positive"));
        }
        payload.build();
    }

    let affected_rows = insert_weather_data_batch(&pool, &payloads).await?;
    let response = json!({
        "inserted": affected_rows,
        "size": payloads.len()
    });
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn route_not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "code":404,
            "message":"route not found"
        }
        )),
    )
}
