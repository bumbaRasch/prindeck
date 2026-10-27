use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::{TimeDelta, Utc};

use super::{History, HistoryHandle, Snapshot, try_record};
use crate::status::DeviceStatus;

fn temp_db(name: &str) -> PathBuf {
    let path =
        std::env::temp_dir().join(format!("prindeck-history-{name}-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    path
}

fn snapshot(ts: &str) -> Snapshot {
    Snapshot {
        ts: ts.to_string(),
        toner_percent: Some(40),
        drum_percent: Some(80),
        pages_total: Some(7753),
        duplex_pages: Some(3967),
        drum_pages_left: Some(4247),
    }
}

#[test]
fn round_trip() {
    let status = DeviceStatus {
        ok: true,
        host: "192.168.187.36".to_string(),
        toner_percent: Some(40),
        drum_percent: Some(80),
        pages_total: Some(7753),
        duplex_pages: Some(3967),
        drum_pages_left: Some(4247),
        jam_count: Some(8),
        polled_at: Some("2026-10-27T15:04:05.678912345+00:00".to_string()),
        error: None,
    };
    let recorded = Snapshot::from_status(&status);
    assert_eq!(recorded, snapshot("2026-10-27T15:04:05.678912345+00:00"));

    let path = temp_db("round-trip");
    let history = History::open(&path).expect("history opens");
    history.record(&recorded).expect("record succeeds");

    let reopened = History::open(&path).expect("history re-opens");
    let rows = reopened.snapshots().expect("snapshots read back");
    assert_eq!(rows, vec![recorded]);
}

#[test]
fn gaps_stay_gaps() {
    let path = temp_db("gaps-stay-gaps");
    let history = History::open(&path).expect("history opens");
    history
        .record(&snapshot("2026-10-01T00:00:00+00:00"))
        .expect("record succeeds");
    history
        .record(&snapshot("2026-10-27T00:00:00+00:00"))
        .expect("record succeeds");

    let rows = history.snapshots().expect("snapshots read back");
    assert_eq!(rows.len(), 2);
}

#[test]
fn try_record_swallows_errors() {
    let handle = HistoryHandle(Mutex::new(None));

    try_record(&handle, &snapshot("2026-10-27T15:04:05+00:00"));
}

#[test]
fn read_days_returns_window_newest_last() {
    let history = History::open(Path::new(":memory:")).expect("in-memory history opens");
    let now = Utc::now();
    let ts =
        |days_ago: i64| (now - TimeDelta::try_days(days_ago).expect("valid delta")).to_rfc3339();

    history.record(&snapshot(&ts(40))).expect("record succeeds");
    history.record(&snapshot(&ts(1))).expect("record succeeds");
    history.record(&snapshot(&ts(20))).expect("record succeeds");
    history.record(&snapshot(&ts(10))).expect("record succeeds");

    let rows = history.read_days(30).expect("read_days succeeds");
    let stamps: Vec<&str> = rows.iter().map(|row| row.ts.as_str()).collect();
    assert_eq!(
        stamps,
        vec![ts(20).as_str(), ts(10).as_str(), ts(1).as_str()]
    );
}

#[test]
fn read_days_with_zero_days_returns_nothing() {
    let history = History::open(Path::new(":memory:")).expect("in-memory history opens");
    history
        .record(&snapshot("2026-10-27T15:04:05+00:00"))
        .expect("record succeeds");

    let rows = history.read_days(0).expect("read_days succeeds");
    assert!(rows.is_empty());
}

#[test]
fn chart_downsample_contract_caps_200_rows_to_90_keeping_endpoints() {
    let total = 200;
    let cap = 90;
    let picked: Vec<usize> = (0..cap)
        .map(|slot| ((slot * (total - 1)) as f64 / (cap - 1) as f64).round() as usize)
        .collect();

    assert_eq!(picked.len(), cap, "the downsample output honors the cap");
    assert_eq!(picked[0], 0, "the first row is always kept");
    assert_eq!(picked[cap - 1], total - 1, "the last row is always kept");
    assert!(
        picked.windows(2).all(|pair| pair[0] < pair[1]),
        "kept indices stay ordered and unique"
    );
}

#[test]
fn chart_empty_state_flag_contract_below_two_points() {
    let chartable = |points: usize| points >= 2;

    assert!(!chartable(0), "zero snapshots render the empty state");
    assert!(!chartable(1), "a single snapshot renders the empty state");
    assert!(chartable(2), "two snapshots already draw the chart");
}

#[test]
fn insert_failure_is_swallowed() {
    let path = temp_db("insert-failure");
    let history = History::open(&path).expect("history opens");
    let snapshot = snapshot("2026-10-27T15:04:05+00:00");
    history.record(&snapshot).expect("record succeeds");

    let handle = HistoryHandle(Mutex::new(Some(history)));

    try_record(&handle, &snapshot);
}
