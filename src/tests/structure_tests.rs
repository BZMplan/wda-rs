use crate::tests::common::sample_upload;

#[test]
fn build_sets_timestamp_when_missing() {
    let mut upload = sample_upload(54511, None);
    upload.build();
    assert!(upload.timestamp.is_some());
}

#[test]
fn build_calculates_dp_and_slp_when_fields_are_complete() {
    let mut upload = sample_upload(54511, None);
    upload.station.station_height = Some(50.0);
    upload.build();
    assert!(upload.weather.dp.is_some());
    assert!(upload.weather.slp.is_some());
}

#[test]
fn build_keeps_dp_and_slp_empty_when_required_fields_missing() {
    let mut upload = sample_upload(54511, None);
    upload.station.station_height = None;
    upload.build();
    assert!(upload.weather.dp.is_none());
    assert!(upload.weather.slp.is_none());
    assert!(upload.timestamp.is_some());
}
