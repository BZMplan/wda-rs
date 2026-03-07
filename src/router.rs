use axum::{
    Json,
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::db::{
    create_weather_table, insert_weather_data_to_table, query_weather_data_from_table, table_exist,
};
use crate::structure::ElemUpload;

#[derive(Deserialize)]
pub struct GetParams {
    station_id: i32,
}

pub async fn get_data_path(Path(params): Path<GetParams>) -> (StatusCode, Json<Value>) {
    let table_name = format!("station_{}", params.station_id);
    let data = query_weather_data_from_table(&table_name).await.unwrap();

    let response = serde_json::to_value(&data).unwrap();

    (StatusCode::OK, Json(response))
}

pub async fn get_data_query(Query(params): Query<GetParams>) -> (StatusCode, Json<Value>) {
    let table_name = format!("station_{}", params.station_id);

    let data = query_weather_data_from_table(&table_name).await.unwrap();

    let response = serde_json::to_value(&data).unwrap();

    (StatusCode::OK, Json(response))
}

pub async fn upload_data(Json(mut playload): Json<ElemUpload>) -> (StatusCode, Json<Value>) {
    playload.build();
    let response = json!({
        "timestamp":playload.timestamp,
        "station":playload.station,
        "weather":playload.weather
    });
    let table_name = format!("station_{}", playload.station.station_id);
    let exist = table_exist(&table_name).await.unwrap();
    if !exist {
        let _result = create_weather_table(&table_name).await;
        print!("{:#?}", _result);
    }
    let _result = insert_weather_data_to_table(&table_name, playload).await;
    print!("{:#?}", _result);
    (StatusCode::OK, Json(response))
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
