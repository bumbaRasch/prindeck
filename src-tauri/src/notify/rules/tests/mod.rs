mod state;

use super::{Measure, NotifyConfig, NotifyKind, evaluate};

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
    evaluate(prev.as_ref(), &next, 0, false, &notify_config())
}

fn evaluate_recovered_step(prev: Measure, next: Measure) -> Vec<NotifyKind> {
    evaluate(Some(&prev), &next, 0, true, &notify_config())
}

fn evaluate_disabled(prev: Option<Measure>, next: Measure, silent_streak: u32) -> Vec<NotifyKind> {
    let mut cfg = notify_config();
    cfg.enabled = false;
    evaluate(prev.as_ref(), &next, silent_streak, true, &cfg)
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
fn unannounced_offline_recovery_fires_nothing() {
    let kinds = evaluate_step(Some(offline_toner(None)), online_toner(Some(70)));
    assert_eq!(kinds, Vec::new());
}

#[test]
fn back_online_does_not_repeat_still_low_toner() {
    let kinds = evaluate_recovered_step(offline_toner(Some(14)), online_toner(Some(14)));
    assert_eq!(kinds, vec![NotifyKind::BackOnline]);
}

#[test]
fn back_online_with_newly_low_toner_fires_both() {
    let kinds = evaluate_recovered_step(offline_toner(Some(20)), online_toner(Some(14)));
    assert_eq!(
        kinds,
        vec![NotifyKind::BackOnline, NotifyKind::TonerLow(14)]
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
