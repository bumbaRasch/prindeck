use std::sync::{Mutex, MutexGuard};

const SILENT_STREAK_POLLS: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotifyConfig {
    pub enabled: bool,
    pub toner_percent: u8,
    pub drum_percent: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Measure {
    pub online: bool,
    pub toner: Option<u8>,
    pub drum: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotifyKind {
    TonerLow(u8),
    DrumLow(u8),
    Silent,
    BackOnline,
}

impl NotifyKind {
    pub fn code(&self) -> &'static str {
        match self {
            NotifyKind::TonerLow(_) => "toner-low",
            NotifyKind::DrumLow(_) => "drum-low",
            NotifyKind::Silent => "silent",
            NotifyKind::BackOnline => "back-online",
        }
    }
}

pub fn evaluate(
    prev: Option<&Measure>,
    next: &Measure,
    silent_streak: u32,
    cfg: &NotifyConfig,
) -> Vec<NotifyKind> {
    if !cfg.enabled {
        return Vec::new();
    }

    let mut kinds = Vec::new();

    if next.online {
        if let Some(prev) = prev
            && !prev.online
        {
            kinds.push(NotifyKind::BackOnline);
        }

        if let Some(percent) = crossing(
            prev.and_then(|measure| measure.toner),
            next.toner,
            cfg.toner_percent,
        ) {
            kinds.push(NotifyKind::TonerLow(percent));
        }

        if let Some(percent) = crossing(
            prev.and_then(|measure| measure.drum),
            next.drum,
            cfg.drum_percent,
        ) {
            kinds.push(NotifyKind::DrumLow(percent));
        }
    } else if silent_streak == SILENT_STREAK_POLLS {
        kinds.push(NotifyKind::Silent);
    }

    kinds
}

fn crossing(prev_value: Option<u8>, next_value: Option<u8>, threshold: u8) -> Option<u8> {
    let value = next_value?;
    let was_below = prev_value.is_some_and(|previous| previous < threshold);
    (value < threshold && !was_below).then_some(value)
}

#[derive(Debug, Default)]
pub struct State {
    snapshot: Mutex<StateSnapshot>,
}

#[derive(Debug, Default)]
struct StateSnapshot {
    prev: Option<Measure>,
    silent_streak: u32,
}

impl State {
    pub fn new() -> State {
        State::default()
    }

    pub fn apply(&self, next: &Measure, cfg: &NotifyConfig) -> Vec<NotifyKind> {
        let mut snapshot = self.lock();
        let silent_streak = if next.online {
            0
        } else {
            snapshot.silent_streak + 1
        };

        let kinds = evaluate(snapshot.prev.as_ref(), next, silent_streak, cfg);

        snapshot.prev = Some(*next);
        snapshot.silent_streak = silent_streak;
        kinds
    }

    fn lock(&self) -> MutexGuard<'_, StateSnapshot> {
        self.snapshot
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
