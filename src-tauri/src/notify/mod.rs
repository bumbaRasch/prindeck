mod rules;

#[cfg(test)]
mod tests;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::commands::AppState;
use crate::status::DeviceStatus;

pub use rules::{Measure, NotifyConfig, NotifyKind, State, evaluate};

pub const EVENT_ALERT: &str = "alert";

const NOTIFY_TONER_TITLE_LABEL: &str = "Тонер заканчивается";
const NOTIFY_TONER_LEVEL_LABEL: &str = "Осталось тонера";
const NOTIFY_DRUM_TITLE_LABEL: &str = "Ресурс барабана заканчивается";
const NOTIFY_DRUM_LEVEL_LABEL: &str = "Осталось ресурса барабана";
const NOTIFY_SILENT_TITLE_LABEL: &str = "Принтер не отвечает";
const NOTIFY_SILENT_BODY_LABEL: &str = "Устройство не отвечает уже на три опроса подряд";
const NOTIFY_BACK_TITLE_LABEL: &str = "Принтер снова доступен";
const NOTIFY_BACK_BODY_LABEL: &str = "Связь с устройством восстановлена";
const NOTIFY_TEST_TITLE_LABEL: &str = "Проверка уведомлений";
const NOTIFY_TEST_BODY_LABEL: &str = "Уведомления работают";

const NOTIFY_TONER_ID: i32 = 1;
const NOTIFY_DRUM_ID: i32 = 2;
const NOTIFY_SILENT_ID: i32 = 3;
const NOTIFY_BACK_ID: i32 = 4;
const NOTIFY_TEST_ID: i32 = 5;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct AlertPayload {
    kind: &'static str,
    message: String,
}

pub fn on_poll(app: &AppHandle, snapshot: &DeviceStatus) {
    let config = app.state::<AppState>().config.get();
    let cfg = rules_config(&config.notify);
    let measure = measure_from_status(snapshot);
    let kinds = app.state::<State>().apply(&measure, &cfg);

    for kind in kinds {
        emit_alert(app, kind);
        show(app, kind);
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
    let (title, body) = notification_text(&kind);
    let payload = AlertPayload {
        kind: kind.code(),
        message: format!("{title}. {body}"),
    };

    if let Err(error) = app.emit(EVENT_ALERT, payload) {
        tracing::debug!(%error, "failed to emit alert");
    }
}

fn show(app: &AppHandle, kind: NotifyKind) {
    let (title, body) = notification_text(&kind);
    let shown = app
        .notification()
        .builder()
        .id(notification_id(&kind))
        .title(title)
        .body(body)
        .show();

    if let Err(error) = shown {
        tracing::warn!(%error, kind = kind.code(), "failed to show a notification");
    }
}

pub fn test_notification_payload() -> (i32, String, String) {
    (
        NOTIFY_TEST_ID,
        NOTIFY_TEST_TITLE_LABEL.to_string(),
        NOTIFY_TEST_BODY_LABEL.to_string(),
    )
}

pub fn show_test(app: &AppHandle) {
    let (id, title, body) = test_notification_payload();
    let shown = app
        .notification()
        .builder()
        .id(id)
        .title(title)
        .body(body)
        .show();

    if let Err(error) = shown {
        tracing::warn!(%error, "failed to show the test notification");
    }
}

fn notification_id(kind: &NotifyKind) -> i32 {
    match kind {
        NotifyKind::TonerLow(_) => NOTIFY_TONER_ID,
        NotifyKind::DrumLow(_) => NOTIFY_DRUM_ID,
        NotifyKind::Silent => NOTIFY_SILENT_ID,
        NotifyKind::BackOnline => NOTIFY_BACK_ID,
    }
}

fn notification_text(kind: &NotifyKind) -> (String, String) {
    match kind {
        NotifyKind::TonerLow(percent) => (
            NOTIFY_TONER_TITLE_LABEL.to_string(),
            format!("{}: {} %", NOTIFY_TONER_LEVEL_LABEL, percent),
        ),
        NotifyKind::DrumLow(percent) => (
            NOTIFY_DRUM_TITLE_LABEL.to_string(),
            format!("{}: {} %", NOTIFY_DRUM_LEVEL_LABEL, percent),
        ),
        NotifyKind::Silent => (
            NOTIFY_SILENT_TITLE_LABEL.to_string(),
            NOTIFY_SILENT_BODY_LABEL.to_string(),
        ),
        NotifyKind::BackOnline => (
            NOTIFY_BACK_TITLE_LABEL.to_string(),
            NOTIFY_BACK_BODY_LABEL.to_string(),
        ),
    }
}
