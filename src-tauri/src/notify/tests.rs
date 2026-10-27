use super::measure_from_status;
use super::rules::{Measure, NotifyConfig, NotifyKind, State, evaluate};
use crate::status::DeviceStatus;

const HOST: &str = "192.168.187.36";

fn notify_config() -> NotifyConfig {
    NotifyConfig {
        enabled: true,
        toner_percent: 15,
        drum_percent: 10,
    }
}

fn online_toner(percent: Option<u8>) -> Measure {
    Measure {
        online: true,
        toner: percent,
        drum: None,
    }
}

fn online_drum(percent: Option<u8>) -> Measure {
    Measure {
        online: true,
        toner: None,
        drum: percent,
    }
}

fn offline_toner(percent: Option<u8>) -> Measure {
    Measure {
        online: false,
        toner: percent,
        drum: None,
    }
}

fn evaluate_step(prev: Option<Measure>, next: Measure) -> Vec<NotifyKind> {
    evaluate(prev.as_ref(), &next, 0, &notify_config())
}

fn evaluate_disabled(prev: Option<Measure>, next: Measure, silent_streak: u32) -> Vec<NotifyKind> {
    let mut cfg = notify_config();
    cfg.enabled = false;
    evaluate(prev.as_ref(), &next, silent_streak, &cfg)
}

fn status(ok: bool, toner: Option<u32>, drum: Option<u32>) -> DeviceStatus {
    DeviceStatus {
        ok,
        host: HOST.to_string(),
        toner_percent: toner,
        drum_percent: drum,
        pages_total: None,
        duplex_pages: None,
        drum_pages_left: None,
        polled_at: None,
        error: None,
    }
}

#[test]
fn toner_threshold_crossing_fires_once() {
    let fired = evaluate_step(Some(online_toner(Some(20))), online_toner(Some(14)));
    assert_eq!(fired, vec![NotifyKind::TonerLow(14)]);

    let steady = evaluate_step(Some(online_toner(Some(14))), online_toner(Some(14)));
    assert_eq!(steady, Vec::new());
}

#[test]
fn toner_rearms_after_recovery() {
    let recovered = evaluate_step(Some(online_toner(Some(14))), online_toner(Some(16)));
    assert_eq!(recovered, Vec::new());

    let crossed_again = evaluate_step(Some(online_toner(Some(16))), online_toner(Some(14)));
    assert_eq!(crossed_again, vec![NotifyKind::TonerLow(14)]);
}

#[test]
fn toner_equal_to_threshold_fires_nothing() {
    let at_threshold = evaluate_step(Some(online_toner(Some(20))), online_toner(Some(15)));
    assert_eq!(
        at_threshold,
        Vec::new(),
        "the PRD notifies strictly below the configured percent"
    );
}

#[test]
fn drum_threshold_crossing_fires_once() {
    let fired = evaluate_step(Some(online_drum(Some(12))), online_drum(Some(8)));
    assert_eq!(fired, vec![NotifyKind::DrumLow(8)]);

    let steady = evaluate_step(Some(online_drum(Some(8))), online_drum(Some(8)));
    assert_eq!(steady, Vec::new());
}

#[test]
fn first_poll_with_low_toner_still_fires() {
    let fired = evaluate_step(None, online_toner(Some(14)));
    assert_eq!(fired, vec![NotifyKind::TonerLow(14)]);
}

#[test]
fn offline_to_online_transition_fires_back_online() {
    let transition = evaluate_step(Some(offline_toner(None)), online_toner(Some(70)));
    assert_eq!(transition, vec![NotifyKind::BackOnline]);

    let first_poll = evaluate_step(None, online_toner(Some(70)));
    assert_eq!(first_poll, Vec::new());
}

#[test]
fn back_online_does_not_repeat_still_low_toner() {
    let kinds = evaluate_step(Some(offline_toner(Some(14))), online_toner(Some(14)));
    assert_eq!(kinds, vec![NotifyKind::BackOnline]);
}

#[test]
fn back_online_with_newly_low_toner_fires_both() {
    let kinds = evaluate_step(Some(offline_toner(Some(20))), online_toner(Some(14)));
    assert_eq!(
        kinds,
        vec![NotifyKind::BackOnline, NotifyKind::TonerLow(14)]
    );
}

#[test]
fn third_failed_poll_fires_silent_once() {
    let state = State::new();
    let cfg = notify_config();

    assert_eq!(state.apply(&offline_toner(None), &cfg), Vec::new());
    assert_eq!(state.apply(&offline_toner(None), &cfg), Vec::new());
    assert_eq!(
        state.apply(&offline_toner(None), &cfg),
        vec![NotifyKind::Silent]
    );
    assert_eq!(state.apply(&offline_toner(None), &cfg), Vec::new());
}

#[test]
fn silent_streak_resets_after_recovery() {
    let state = State::new();
    let cfg = notify_config();

    state.apply(&offline_toner(None), &cfg);
    state.apply(&offline_toner(None), &cfg);
    assert_eq!(
        state.apply(&online_toner(Some(70)), &cfg),
        vec![NotifyKind::BackOnline]
    );

    assert_eq!(state.apply(&offline_toner(None), &cfg), Vec::new());
    assert_eq!(state.apply(&offline_toner(None), &cfg), Vec::new());
    assert_eq!(
        state.apply(&offline_toner(None), &cfg),
        vec![NotifyKind::Silent]
    );
}

#[test]
fn disabled_config_suppresses_every_kind() {
    let toner = evaluate_disabled(Some(online_toner(Some(20))), online_toner(Some(14)), 0);
    assert_eq!(toner, Vec::new());

    let silent = evaluate_disabled(Some(offline_toner(None)), offline_toner(None), 3);
    assert_eq!(silent, Vec::new());

    let back = evaluate_disabled(Some(offline_toner(None)), online_toner(Some(70)), 0);
    assert_eq!(back, Vec::new());
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
