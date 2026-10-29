use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};

use super::{DeviceStatus, EVENT_STATUS, StatusCache};
use crate::commands::AppState;
use crate::config::ConfigHandle;
use crate::history::{HistoryHandle, Snapshot, try_record};
use crate::notify;
use crate::snmp::SnmpClient;

const RETRY_DELAY: Duration = Duration::from_secs(2);

pub fn start_polling(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            if !poll_now(app.clone()).await.ok {
                tokio::time::sleep(RETRY_DELAY).await;
                poll_now(app.clone()).await;
            }

            let delay = next_delay(&app.state::<AppState>().config);
            tokio::time::sleep(delay).await;
        }
    });
}

pub(super) fn next_delay(handle: &ConfigHandle) -> Duration {
    handle.get().poll_interval()
}

pub async fn poll_now(app: AppHandle) -> DeviceStatus {
    let (host, community) = {
        let config = app.state::<AppState>().config.get();
        (
            config.device.host.clone(),
            config.snmp_community().to_owned(),
        )
    };
    let cache = app.state::<StatusCache>();
    let client = SnmpClient::new(&host, &community);

    let fetched = tauri::async_runtime::spawn_blocking(move || client.fetch())
        .await
        .map_err(|error| error.to_string())
        .and_then(|result| result.map_err(|error| error.to_string()));

    match fetched {
        Ok(counters) => cache.apply_success(counters),
        Err(error) => {
            tracing::warn!(%error, "snmp poll failed");
            cache.apply_failure(error);
        }
    }

    let snapshot = cache.snapshot();
    if snapshot.ok
        && let Some(handle) = app.try_state::<HistoryHandle>()
    {
        try_record(&handle, &Snapshot::from_status(&snapshot));
    }
    emit_status(&app, &snapshot);
    notify::on_poll(&app, &snapshot);
    #[cfg(desktop)]
    crate::tray::refresh(&app);
    snapshot
}

fn emit_status(app: &AppHandle, snapshot: &DeviceStatus) {
    if let Err(error) = app.emit(EVENT_STATUS, snapshot.clone()) {
        tracing::debug!(%error, "failed to emit status");
    }
}
