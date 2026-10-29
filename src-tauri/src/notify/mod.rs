mod rules;

#[cfg(test)]
mod tests;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::commands::AppState;
use crate::status::DeviceStatus;

pub use rules::{Measure, NotifyConfig, NotifyKind, State, evaluate};

pub const EVENT_ALERT: &str = "alert";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct AlertPayload {
    kind: String,
    percent: Option<u8>,
}

pub fn on_poll(app: &AppHandle, snapshot: &DeviceStatus) {
    let config = app.state::<AppState>().config.get();
    let cfg = rules_config(&config.notify);
    let measure = measure_from_status(snapshot);
    let kinds = app.state::<State>().apply(&measure, &cfg);

    for kind in kinds {
        emit_alert(app, kind);
    }
}

pub fn rearm(app: &AppHandle) {
    app.state::<State>().rearm();
}

fn rules_config(config: &crate::config::NotifyConfig) -> NotifyConfig {
    NotifyConfig {
        enabled: config.enabled,
        toner_percent: config.toner_percent,
        drum_percent: config.drum_percent,
    }
}

fn measure_from_status(status: &DeviceStatus) -> Measure {
    Measure {
        online: status.ok,
        toner: percent(status.toner_percent),
        drum: percent(status.drum_percent),
    }
}

fn percent(value: Option<u32>) -> Option<u8> {
    value.and_then(|value| u8::try_from(value).ok())
}

fn emit_alert(app: &AppHandle, kind: NotifyKind) {
    let payload = alert_payload(kind);
    tracing::debug!(
        kind = payload.kind.as_str(),
        percent = ?payload.percent,
        "alert emitted"
    );

    if let Err(error) = app.emit(EVENT_ALERT, payload) {
        tracing::debug!(%error, "failed to emit alert");
    }
}

fn alert_payload(kind: NotifyKind) -> AlertPayload {
    let percent = match kind {
        NotifyKind::TonerLow(value) | NotifyKind::DrumLow(value) => Some(value),
        NotifyKind::Silent | NotifyKind::BackOnline => None,
    };

    AlertPayload {
        kind: kind.code().to_string(),
        percent,
    }
}
