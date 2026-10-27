use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use super::AppState;
use crate::status::{self, DeviceStatus};

#[tauri::command]
pub fn get_status(app: AppHandle) -> DeviceStatus {
    status::cached(&app)
}

#[tauri::command]
pub async fn refresh_status(app: AppHandle) -> Result<DeviceStatus, String> {
    let snapshot = status::poll_now(app).await;

    if !snapshot.ok && snapshot.polled_at.is_none() {
        Err(snapshot
            .error
            .unwrap_or_else(|| "the device is not responding".to_string()))
    } else {
        Ok(snapshot)
    }
}

#[tauri::command]
pub fn open_printer_panel(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let url = format!("http://{}/", state.config.get().device.host);

    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|error| format!("failed to open the printer panel: {error}"))
}
