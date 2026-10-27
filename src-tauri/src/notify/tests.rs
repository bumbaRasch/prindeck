use super::{measure_from_status, test_notification_payload};
use crate::status::DeviceStatus;

const HOST: &str = "192.168.187.36";

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
fn test_notification_payload_is_fixed() {
    let (id, title, body) = test_notification_payload();

    assert_eq!(id, super::NOTIFY_TEST_ID);
    assert_eq!(title, super::NOTIFY_TEST_TITLE_LABEL.to_string());
    assert_eq!(body, super::NOTIFY_TEST_BODY_LABEL.to_string());
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
