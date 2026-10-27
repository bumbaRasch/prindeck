use super::icons::{TrayIconState, TrayState, icon_state};
use crate::commands::{ACTIVITY_IDLE, ACTIVITY_PRINTING, ACTIVITY_SCANNING};
use crate::status::DeviceStatus;

#[test]
fn offline_wins_over_any_activity() {
    for activity in [ACTIVITY_IDLE, ACTIVITY_SCANNING, ACTIVITY_PRINTING] {
        assert_eq!(
            icon_state(TrayState::Offline, activity),
            TrayIconState::Offline
        );
    }
}

#[test]
fn error_beats_idle_and_working_activities() {
    assert_eq!(
        icon_state(TrayState::Error, ACTIVITY_IDLE),
        TrayIconState::Error
    );
    assert_eq!(
        icon_state(TrayState::Error, ACTIVITY_SCANNING),
        TrayIconState::Error
    );
}

#[test]
fn ok_with_scanning_activity_is_the_working_icon() {
    assert_eq!(
        icon_state(TrayState::Ok, ACTIVITY_SCANNING),
        TrayIconState::Working
    );
    assert_eq!(
        icon_state(TrayState::Ok, ACTIVITY_PRINTING),
        TrayIconState::Working
    );
}

#[test]
fn ok_with_idle_activity_is_the_idle_icon() {
    assert_eq!(
        icon_state(TrayState::Ok, ACTIVITY_IDLE),
        TrayIconState::Idle
    );
}

#[test]
fn device_status_maps_to_ok_and_offline() {
    assert_eq!(TrayState::from_status(&status(true)), TrayState::Ok);
    assert_eq!(TrayState::from_status(&status(false)), TrayState::Offline);
}

#[test]
fn navigate_payload_is_the_bare_scan_screen_literal() {
    assert_eq!(super::SCAN_SCREEN, "scan");
}

fn status(ok: bool) -> DeviceStatus {
    DeviceStatus {
        ok,
        host: "192.168.187.36".to_string(),
        toner_percent: None,
        drum_percent: None,
        jam_count: None,
        pages_total: None,
        duplex_pages: None,
        drum_pages_left: None,
        polled_at: None,
        error: None,
    }
}
