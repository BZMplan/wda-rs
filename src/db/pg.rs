use crate::db::DB_POOL;
use sqlx::Executor;

use crate::structure::ElemUpload;

pub async fn table_exist(table_name: &str) -> Result<bool, sqlx::Error> {
    let exists: Option<String> = sqlx::query_scalar("SELECT to_regclass($1)::text")
        .bind(table_name)
        .fetch_one(&*DB_POOL)
        .await?;
    Ok(exists.is_some())
}

pub async fn create_weather_table(table_name: &str) -> Result<(), sqlx::Error> {
    let sql = format!(
        r#"
            CREATE TABLE IF NOT EXISTS {} (
                id BIGSERIAL PRIMARY KEY,
                timestamp TIMESTAMP NOT NULL,
                station_id BIGINT NOT NULL,
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
                wd BIGINT
            )
      "#,
        table_name
    );

    DB_POOL.execute(sql.as_str()).await?;

    Ok(())
}

pub async fn insert_weather_data_to_table(
    table_name: &str,
    upload: ElemUpload,
) -> Result<(), sqlx::Error> {
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

    sqlx::query(&sql)
        .bind(upload.timestamp)
        .bind(upload.station.station_id)
        .bind(upload.station.station_name)
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
        .execute(&*DB_POOL)
        .await?;

    Ok(())
}
