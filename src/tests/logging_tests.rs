use chrono::{Local, TimeZone};
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing_subscriber::fmt::MakeWriter;

use crate::{RotatingLatestLogger, format_date};

fn temp_log_dir() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    env::temp_dir().join(format!("wda_rs_log_test_{unique}"))
}

#[test]
fn format_date_outputs_yyyy_mm_dd() {
    let date = Local
        .with_ymd_and_hms(2026, 3, 11, 12, 0, 0)
        .single()
        .expect("date should be constructible");
    assert_eq!(format_date(date), "2026-03-11");
}

#[test]
fn logger_writes_to_latest_and_rotates_to_dated_file() {
    let dir = temp_log_dir();

    {
        let logger = RotatingLatestLogger::new(&dir).expect("logger should initialize");
        {
            let mut writer = logger.make_writer();
            writer
                .write_all(b"first-line\n")
                .expect("first write should succeed");
            writer.flush().expect("first flush should succeed");
        }

        let initial_latest = fs::read_to_string(dir.join("latest.log"))
            .expect("latest.log should exist after first write");
        assert!(initial_latest.contains("first-line"));

        {
            let mut state = logger
                .inner
                .lock()
                .expect("logger state should be lockable");
            state.current_date = "2000-01-01".to_string();
        }

        {
            let mut writer = logger.make_writer();
            writer
                .write_all(b"second-line\n")
                .expect("second write should succeed");
            writer.flush().expect("second flush should succeed");
        }

        let archived = fs::read_to_string(dir.join("2000-01-01.log"))
            .expect("archived date log should exist after rotation");
        let latest = fs::read_to_string(dir.join("latest.log"))
            .expect("latest.log should exist after rotation");

        assert!(archived.contains("first-line"));
        assert!(latest.contains("second-line"));
        assert!(!latest.contains("first-line"));
    }

    fs::remove_dir_all(&dir).expect("temporary test directory should be removable");
}
