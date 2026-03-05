use axum::{
    Json,
    extract::{Path, Query},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::db::{create_weather_table, insert_weather_data_to_table, table_exist};
use crate::structure::ElemUpload;

#[derive(Deserialize)]
pub struct UserParams {
    param_type: String,
    param: i32,
}

pub async fn get_data(Path(params): Path<UserParams>) -> (StatusCode, Json<Value>) {
    let response = json!({
        "param_type":params.param_type,
        "param":params.param
    });
    (StatusCode::OK, Json(response))
}

pub async fn get_data_query(Query(params): Query<UserParams>) -> (StatusCode, Json<Value>) {
    let response = json!({
        "param_type":params.param_type,
        "param":params.param
    });
    (StatusCode::OK, Json(response))
}

pub async fn upload_data(Json(mut playload): Json<ElemUpload>) -> (StatusCode, Json<Value>) {
    playload.build();
    let response = json!({
        "station":playload.station,
        "weather":playload.weather
    });
    let exist = table_exist("test").await.unwrap();
    if !exist {
        let _result = create_weather_table("test").await;
        // print!("{:#?}",a);
    }
    let _result = insert_weather_data_to_table("test", playload).await;
    // print!("{:#?}",a);
    (StatusCode::OK, Json(response))
}
