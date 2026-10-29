use super::{NotifyKind, alert_payload, measure_from_status};
use crate::status::DeviceStatus;

const HOST: &str = "192.0.2.10";

fn status(ok: bool, toner: Option<u32>, drum: Option<u32>) -> DeviceStatus {
    DeviceStatus {
        ok,
        host: HOST.to_string(),
        toner_percent: toner,
        drum_percent: drum,
        jam_count: None,
        pages_total: None,
        duplex_pages: None,
        drum_pages_left: None,
        polled_at: None,
        error: None,
    }
}

#[test]
fn alert_payload_carries_the_kind_code_and_percent() {
    let toner = alert_payload(NotifyKind::TonerLow(14));
    assert_eq!(
        (toner.kind.as_str(), toner.percent),
        ("toner-low", Some(14))
    );

    let drum = alert_payload(NotifyKind::DrumLow(8));
    assert_eq!((drum.kind.as_str(), drum.percent), ("drum-low", Some(8)));

    let silent = alert_payload(NotifyKind::Silent);
    assert_eq!((silent.kind.as_str(), silent.percent), ("silent", None));

    let back = alert_payload(NotifyKind::BackOnline);
    assert_eq!((back.kind.as_str(), back.percent), ("back-online", None));
}

#[test]
fn alert_payload_serializes_kind_and_percent_only() {
    let toner = serde_json::to_value(alert_payload(NotifyKind::TonerLow(14))).unwrap();
    assert_eq!(
        toner,
        serde_json::json!({"kind": "toner-low", "percent": 14})
    );

    let silent = serde_json::to_value(alert_payload(NotifyKind::Silent)).unwrap();
    assert_eq!(
        silent,
        serde_json::json!({"kind": "silent", "percent": null})
    );
}

#[test]
fn measure_maps_the_status_snapshot() {
    let online = measure_from_status(&status(true, Some(14), Some(8)));
    assert_eq!(
        (online.online, online.toner, online.drum),
        (true, Some(14), Some(8))
    );

    let offline = measure_from_status(&status(false, Some(14), Some(8)));
    assert_eq!(
        (offline.online, offline.toner, offline.drum),
        (false, Some(14), Some(8))
    );

    let clamped = measure_from_status(&status(true, Some(300), None));
    assert_eq!(
        (clamped.online, clamped.toner, clamped.drum),
        (true, None, None)
    );
}
