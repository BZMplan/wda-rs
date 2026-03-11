use chrono::Utc;
use sqlx::{Executor, PgPool};

use crate::structure::{ElemGet, ElemUpload};

fn station_table(station_id: i32) -> String {
    format!("station_{station_id}")
}

pub async fn table_exist(pool: &PgPool, station_id: i32) -> Result<bool, sqlx::Error> {
    let table_name = station_table(station_id);
    let exists: Option<String> = sqlx::query_scalar("SELECT to_regclass($1)::text")
        .bind(table_name)
        .fetch_one(pool)
        .await?;
    Ok(exists.is_some())
}

pub async fn create_weather_table(pool: &PgPool, station_id: i32) -> Result<(), sqlx::Error> {
    let table_name = station_table(station_id);
    let sql = format!(
        r#"
            CREATE TABLE IF NOT EXISTS {} (
                id BIGSERIAL PRIMARY KEY,
                timestamp BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM now()),
                station_id INTEGER NOT NULL,
                station_name TEXT,
                station_lon DOUBLE PRECISION,
                station_lat DOUBLE PRECISION,
                station_height DOUBLE PRECISION,
                t DOUBLE PRECISION,
                p DOUBLE PRECISION,
                rh DOUBLE PRECISION,
                dp DOUBLE PRECISION,
                slp DOUBLE PRECISION,
                ws DOUBLE PRECISION,
                wd INTEGER
            )
      "#,
        table_name
    );

    pool.execute(sql.as_str()).await?;

    Ok(())
}

pub async fn insert_weather_data_to_table(
    pool: &PgPool,
    station_id: i32,
    upload: &ElemUpload,
) -> Result<(), sqlx::Error> {
    let table_name = station_table(station_id);
    let sql = format!(
        r#"
        INSERT INTO {} (
            timestamp,station_id,station_name,station_lon,station_lat,station_height,
            t,p,rh,dp,slp,ws,wd
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
        "#,
        table_name
    );
    let timestamp = upload.timestamp.unwrap_or_else(|| Utc::now().timestamp());

    sqlx::query(&sql)
        .bind(timestamp)
        .bind(upload.station.station_id)
        .bind(&upload.station.station_name)
        .bind(upload.station.station_lon)
        .bind(upload.station.station_lat)
        .bind(upload.station.station_height)
        .bind(upload.weather.t)
        .bind(upload.weather.p)
        .bind(upload.weather.rh)
        .bind(upload.weather.dp)
        .bind(upload.weather.slp)
        .bind(upload.weather.ws)
        .bind(upload.weather.wd)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn query_weather_data_from_table(
    pool: &PgPool,
    station_id: i32,
) -> Result<Option<ElemGet>, sqlx::Error> {
    let table_name = station_table(station_id);
    let sql = format!(
        r#"
        SELECT * FROM {} ORDER BY timestamp DESC LIMIT 1
        "#,
        table_name
    );

    let row = sqlx::query_as::<_, ElemGet>(&sql)
        .fetch_optional(pool)
        .await?;

    Ok(row)
}
