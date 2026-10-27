use std::path::PathBuf;
use std::sync::Mutex;

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
