use crate::db::insert_weather_data_batch;
use crate::db::pg::{INIT_STATION_TIME_INDEX_SQL, QUERY_LATEST_SQL, effective_timestamp};
use crate::tests::common::{lazy_pool, sample_upload};

#[test]
fn effective_timestamp_keeps_existing_value() {
    let upload = sample_upload(54511, Some(1_700_000_000));
    assert_eq!(effective_timestamp(&upload), 1_700_000_000);
}

#[test]
fn query_sql_contains_station_filter_and_latest_order() {
    assert!(QUERY_LATEST_SQL.contains("WHERE station_id = $1"));
    assert!(QUERY_LATEST_SQL.contains("ORDER BY observed_at DESC"));
    assert!(INIT_STATION_TIME_INDEX_SQL.contains("station_id, observed_at DESC"));
}

#[tokio::test]
async fn insert_batch_returns_zero_for_empty_payload() {
    let pool = lazy_pool();
    let inserted = insert_weather_data_batch(&pool, &[])
        .await
        .expect("empty payload should return without querying db");
    assert_eq!(inserted, 0);
}
