use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicU8;
use std::sync::{Arc, Mutex, MutexGuard, RwLock};

use reqwest::Url;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::config::{Config, ConfigHandle};
use crate::escl::{EsclClient, ScanCaps};
use caps::resolve_state;

mod activity;
mod caps;
mod print;
mod scan;
mod scan_errors;
mod scan_files;
mod scan_jobs;
mod scan_pages;
mod settings;
mod status;

pub const EVENT_PROGRESS: &str = "scan-progress";
pub const EVENT_DONE: &str = "scan-done";
pub const EVENT_ERROR: &str = "scan-error";

const FALLBACK_BASE_URL: &str = "http://127.0.0.1:9/eSCL/";

#[derive(Debug)]
pub struct AppState {
    pub config: ConfigHandle,
    pub activity: Arc<AtomicU8>,
    client: RwLock<EsclClient>,
    caps: Mutex<Option<ScanCaps>>,
    jobs: Mutex<HashMap<String, Url>>,
    startup_error: Option<String>,
}

impl AppState {
    pub fn initialize() -> AppState {
        let (config, client, startup_error) = resolve_state();
        AppState::from_parts(config, client, startup_error)
    }

    pub(super) fn from_parts(
        config: Config,
        client: EsclClient,
        startup_error: Option<String>,
    ) -> AppState {
        AppState {
            config: ConfigHandle::new(config),
            activity: Arc::new(AtomicU8::new(activity::ACTIVITY_IDLE)),
            client: RwLock::new(client),
            caps: Mutex::new(None),
            jobs: Mutex::new(HashMap::new()),
            startup_error,
        }
    }

    pub fn client(&self) -> EsclClient {
        self.client
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}

fn lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>, String> {
    Ok(mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ScanPhase {
    Created,
    Scanning,
    Done,
    Canceled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    pub job_id: String,
    pub phase: ScanPhase,
    pub pages: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanDone {
    pub job_id: String,
    pub files: Vec<String>,
    pub partial: bool,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanError {
    pub job_id: String,
    pub kind: ScanErrorKind,
    pub message: String,
}

fn emit_progress(app: &AppHandle, job_id: &str, phase: ScanPhase, pages: u32) {
    let payload = ScanProgress {
        job_id: job_id.to_string(),
        phase,
        pages,
    };

    if let Err(error) = app.emit(EVENT_PROGRESS, payload) {
        tracing::debug!(%error, "failed to emit scan-progress");
    }
}

fn emit_done(
    app: &AppHandle,
    job_id: &str,
    files: &[PathBuf],
    partial: bool,
    warning: Option<String>,
) {
    let payload = ScanDone {
        job_id: job_id.to_string(),
        files: files
            .iter()
            .map(|file| file.display().to_string())
            .collect(),
        partial,
        warning,
    };

    if let Err(error) = app.emit(EVENT_DONE, payload) {
        tracing::debug!(%error, "failed to emit scan-done");
    }
}

fn emit_error(app: &AppHandle, job_id: &str, failure: ScanFailure) {
    let payload = ScanError {
        job_id: job_id.to_string(),
        kind: failure.kind,
        message: failure.message,
    };

    if let Err(error) = app.emit(EVENT_ERROR, payload) {
        tracing::debug!(%error, "failed to emit scan-error");
    }
}

pub use activity::{ACTIVITY_IDLE, ACTIVITY_PRINTING, ACTIVITY_SCANNING};
pub use caps::{get_capabilities, get_startup_error, open_scan_folder};
pub use print::{cancel_print, get_queue, print_file};
pub use scan::{cancel_scan, start_scan};
pub use scan_errors::{ScanErrorKind, ScanFailure, user_message};
pub use settings::{get_config, set_config};
pub use status::{get_status, open_printer_panel, open_status_page, refresh_status};

pub use caps::{
    __cmd__get_capabilities, __cmd__get_startup_error, __cmd__open_scan_folder,
    __tauri_command_name_get_capabilities, __tauri_command_name_get_startup_error,
    __tauri_command_name_open_scan_folder,
};
pub use print::{
    __cmd__cancel_print, __cmd__get_queue, __cmd__print_file, __tauri_command_name_cancel_print,
    __tauri_command_name_get_queue, __tauri_command_name_print_file,
};
pub use scan::{
    __cmd__cancel_scan, __cmd__start_scan, __tauri_command_name_cancel_scan,
    __tauri_command_name_start_scan,
};
pub use settings::{
    __cmd__get_config, __cmd__set_config, __tauri_command_name_get_config,
    __tauri_command_name_set_config,
};
pub use status::{
    __cmd__get_status, __cmd__open_printer_panel, __cmd__refresh_status,
    __tauri_command_name_get_status, __tauri_command_name_open_printer_panel,
    __tauri_command_name_refresh_status,
};

#[cfg(test)]
mod tests;
