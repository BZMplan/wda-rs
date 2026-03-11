use crate::structure::{ElemUpload, Station, Weather};
use sqlx::postgres::PgPoolOptions;

pub fn lazy_pool() -> sqlx::PgPool {
    PgPoolOptions::new()
        .connect_lazy("postgres://user:password@localhost/test_db")
        .expect("lazy pool should be created")
}

pub fn sample_upload(station_id: i32, timestamp: Option<i64>) -> ElemUpload {
    ElemUpload {
        station: Station {
            station_id,
            station_name: Some("TestStation".to_string()),
            station_height: Some(10.0),
            station_lat: Some(39.9),
            station_lon: Some(116.4),
        },
        weather: Weather {
            t: Some(20.0),
            p: Some(1000.0),
            rh: Some(60.0),
            dp: None,
            slp: None,
            ws: Some(3.0),
            wd: Some(180),
        },
        timestamp,
    }
}
