use super::super::{NotifyKind, State};
use super::{notify_config, offline_toner, online_toner};

fn fail(state: &State) -> Vec<NotifyKind> {
    state.apply(&offline_toner(None), &notify_config())
}

fn succeed(state: &State, toner_percent: u8) -> Vec<NotifyKind> {
    state.apply(&online_toner(Some(toner_percent)), &notify_config())
}

#[test]
fn single_failed_poll_recovers_silently() {
    let state = State::new();

    assert_eq!(fail(&state), Vec::new());
    assert_eq!(succeed(&state, 70), Vec::new());
}

#[test]
fn recovery_after_two_failed_polls_fires_nothing() {
    let state = State::new();

    assert_eq!(fail(&state), Vec::new());
    assert_eq!(fail(&state), Vec::new());
    assert_eq!(succeed(&state, 70), Vec::new());
}

#[test]
fn full_offline_episode_fires_silent_once_then_back_online() {
    let state = State::new();

    assert_eq!(fail(&state), Vec::new());
    assert_eq!(fail(&state), Vec::new());
    assert_eq!(fail(&state), vec![NotifyKind::Silent]);
    assert_eq!(fail(&state), Vec::new(), "silent fires once per episode");
    assert_eq!(succeed(&state, 70), vec![NotifyKind::BackOnline]);
}

#[test]
fn second_offline_episode_fires_both_events_again() {
    let state = State::new();

    for _ in 0..3 {
        fail(&state);
    }
    succeed(&state, 70);

    assert_eq!(fail(&state), Vec::new());
    assert_eq!(fail(&state), Vec::new());
    assert_eq!(fail(&state), vec![NotifyKind::Silent]);
    assert_eq!(succeed(&state, 70), vec![NotifyKind::BackOnline]);
}

#[test]
fn raising_threshold_above_current_level_warns_once_after_rearm() {
    let state = State::new();
    assert_eq!(succeed(&state, 40), Vec::new());

    let mut raised = notify_config();
    raised.toner_percent = 45;
    assert_eq!(
        state.apply(&online_toner(Some(40)), &raised),
        Vec::new(),
        "without rearm the previous measure already sits below the new threshold"
    );

    state.rearm();
    assert_eq!(
        state.apply(&online_toner(Some(40)), &raised),
        vec![NotifyKind::TonerLow(40)]
    );
    assert_eq!(
        state.apply(&online_toner(Some(40)), &raised),
        Vec::new(),
        "the crossing memory is rebuilt from the current level"
    );
}

#[test]
fn enabling_below_threshold_warns_once_after_rearm() {
    let state = State::new();
    let mut disabled = notify_config();
    disabled.enabled = false;

    assert_eq!(state.apply(&online_toner(Some(9)), &disabled), Vec::new());
    assert_eq!(
        state.apply(&online_toner(Some(9)), &notify_config()),
        Vec::new(),
        "without rearm the earlier measure suppresses the first warning"
    );

    state.rearm();
    assert_eq!(
        state.apply(&online_toner(Some(9)), &notify_config()),
        vec![NotifyKind::TonerLow(9)]
    );
    assert_eq!(
        state.apply(&online_toner(Some(9)), &notify_config()),
        Vec::new()
    );
}

#[test]
fn rearm_keeps_the_offline_episode_memory() {
    let state = State::new();

    assert_eq!(fail(&state), Vec::new());
    assert_eq!(fail(&state), Vec::new());
    assert_eq!(fail(&state), vec![NotifyKind::Silent]);

    state.rearm();
    assert_eq!(
        fail(&state),
        Vec::new(),
        "rearm must not repeat the silent warning"
    );
    assert_eq!(
        succeed(&state, 70),
        vec![NotifyKind::BackOnline],
        "the announced offline flag survives the rearm"
    );
}
