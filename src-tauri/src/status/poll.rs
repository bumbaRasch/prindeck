use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};

use super::{DeviceStatus, EVENT_STATUS, StatusCache};
use crate::commands::AppState;
use crate::snmp::SnmpClient;

const RETRY_DELAY: Duration = Duration::from_secs(2);

pub fn start_polling(app: AppHandle) {
    let interval = app.state::<AppState>().config.poll_interval();

    tauri::async_runtime::spawn(async move {
        let mut ticker = tokio::time::interval(interval);

        loop {
            ticker.tick().await;

            if !poll_now(app.clone()).await.ok {
                tokio::time::sleep(RETRY_DELAY).await;
                poll_now(app.clone()).await;
            }
        }
    });
}

pub async fn poll_now(app: AppHandle) -> DeviceStatus {
    let (host, community) = {
        let state = app.state::<AppState>();
        (
            state.config.device.host.clone(),
            state.config.snmp_community().to_owned(),
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
    emit_status(&app, &snapshot);
    snapshot
}

fn emit_status(app: &AppHandle, snapshot: &DeviceStatus) {
    if let Err(error) = app.emit(EVENT_STATUS, snapshot.clone()) {
        tracing::debug!(%error, "failed to emit status");
    }
}
