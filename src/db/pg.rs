use chrono::Utc;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::structure::{ElemGet, ElemUpload};

const INIT_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS weather_data (
    id BIGSERIAL PRIMARY KEY,
    station_id INTEGER NOT NULL,
    station_name TEXT,
    station_lon DOUBLE PRECISION,
    station_lat DOUBLE PRECISION,
    station_height DOUBLE PRECISION,
    observed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    t DOUBLE PRECISION,
    p DOUBLE PRECISION,
    rh DOUBLE PRECISION,
    dp DOUBLE PRECISION,
    slp DOUBLE PRECISION,
    ws DOUBLE PRECISION,
    wd INTEGER
)
"#;

pub(crate) const INIT_STATION_TIME_INDEX_SQL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_weather_data_station_observed_at
ON weather_data (station_id, observed_at DESC)
"#;

const INIT_TIME_BRIN_INDEX_SQL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_weather_data_observed_at_brin
ON weather_data
USING BRIN (observed_at)
"#;

const INSERT_ONE_SQL: &str = r#"
INSERT INTO weather_data (
    observed_at, station_id, station_name, station_lon, station_lat, station_height,
    t, p, rh, dp, slp, ws, wd
)
VALUES (
    to_timestamp($1::double precision), $2, $3, $4, $5, $6,
    $7, $8, $9, $10, $11, $12, $13
)
"#;

pub(crate) const QUERY_LATEST_SQL: &str = r#"
SELECT
    station_id,
    station_name,
    station_height,
    station_lat,
    station_lon,
    EXTRACT(EPOCH FROM observed_at)::BIGINT AS timestamp,
    t,
    p,
    rh,
    dp,
    slp,
    ws,
    wd
FROM weather_data
WHERE station_id = $1
ORDER BY observed_at DESC
LIMIT 1
"#;

pub(crate) fn effective_timestamp(upload: &ElemUpload) -> i64 {
    upload.timestamp.unwrap_or_else(|| Utc::now().timestamp())
}

pub async fn init_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(INIT_TABLE_SQL).execute(pool).await?;
    sqlx::query(INIT_STATION_TIME_INDEX_SQL)
        .execute(pool)
        .await?;
    sqlx::query(INIT_TIME_BRIN_INDEX_SQL).execute(pool).await?;
    Ok(())
}

pub async fn insert_weather_data(pool: &PgPool, upload: &ElemUpload) -> Result<(), sqlx::Error> {
    let timestamp = effective_timestamp(upload) as f64;

    sqlx::query(INSERT_ONE_SQL)
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

pub async fn insert_weather_data_batch(
    pool: &PgPool,
    uploads: &[ElemUpload],
) -> Result<u64, sqlx::Error> {
    if uploads.is_empty() {
        return Ok(0);
    }

    let mut builder = QueryBuilder::<Postgres>::new(
        "INSERT INTO weather_data (observed_at, station_id, station_name, station_lon, station_lat, station_height, t, p, rh, dp, slp, ws, wd) ",
    );

    builder.push_values(uploads, |mut b, upload| {
        b.push("to_timestamp(")
            .push_bind(effective_timestamp(upload) as f64)
            .push(")")
            .push_bind(upload.station.station_id)
            .push_bind(&upload.station.station_name)
            .push_bind(upload.station.station_lon)
            .push_bind(upload.station.station_lat)
            .push_bind(upload.station.station_height)
            .push_bind(upload.weather.t)
            .push_bind(upload.weather.p)
            .push_bind(upload.weather.rh)
            .push_bind(upload.weather.dp)
            .push_bind(upload.weather.slp)
            .push_bind(upload.weather.ws)
            .push_bind(upload.weather.wd);
    });

    let result = builder.build().execute(pool).await?;
    Ok(result.rows_affected())
}

pub async fn query_latest_weather_data(
    pool: &PgPool,
    station_id: i32,
) -> Result<Option<ElemGet>, sqlx::Error> {
    let row = sqlx::query_as::<_, ElemGet>(QUERY_LATEST_SQL)
        .bind(station_id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}
