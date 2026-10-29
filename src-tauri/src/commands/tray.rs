use tauri::AppHandle;

use crate::tray::{TrayLabels, apply_labels};

#[tauri::command]
pub fn set_tray_labels(app: AppHandle, labels: TrayLabels) -> Result<(), String> {
    apply_labels(&app, &labels).map_err(|error| error.to_string())
}
