use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

use super::AppState;
use crate::status::{self, DeviceStatus};

const PANEL_STATUS_PATH: &str = "/general/status.html";
const FIRMWARE_PATH: &str = "/admin/firmwareupdate.html";

fn open_device_page(app: &AppHandle, path: &str) -> Result<(), String> {
    let host = app.state::<AppState>().config.get().device.host;
    let url = format!("http://{host}{path}");

    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|error| format!("failed to open the printer panel: {error}"))
}

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
pub fn open_printer_panel(app: AppHandle) -> Result<(), String> {
    open_device_page(&app, "/")
}

#[tauri::command]
pub fn open_firmware_page(app: AppHandle) -> Result<(), String> {
    open_device_page(&app, FIRMWARE_PATH)
}

pub fn open_status_page(app: &AppHandle) -> Result<(), String> {
    open_device_page(app, PANEL_STATUS_PATH)
}
