use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use reqwest::Url;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::config::Config;
use crate::escl::{EsclClient, EsclError, ScanCaps};
use caps::resolve_state;

mod caps;
mod scan;
mod scan_files;
mod scan_jobs;

pub const EVENT_PROGRESS: &str = "scan-progress";
pub const EVENT_DONE: &str = "scan-done";
pub const EVENT_ERROR: &str = "scan-error";

const FALLBACK_BASE_URL: &str = "http://127.0.0.1:9/eSCL/";

#[derive(Debug)]
pub struct AppState {
    pub config: Config,
    pub client: EsclClient,
    caps: Mutex<Option<ScanCaps>>,
    jobs: Mutex<HashMap<String, Url>>,
    startup_error: Option<String>,
}

impl AppState {
    pub fn initialize() -> AppState {
        let (config, client, startup_error) = resolve_state();

        AppState {
            config,
            client,
            caps: Mutex::new(None),
            jobs: Mutex::new(HashMap::new()),
            startup_error,
        }
    }
}

fn lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>, String> {
    mutex
        .lock()
        .map_err(|_| "internal error: the state is locked after a panic".to_string())
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
    pub file: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanError {
    pub job_id: String,
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

fn emit_done(app: &AppHandle, job_id: &str, file: &Path) {
    let payload = ScanDone {
        job_id: job_id.to_string(),
        file: file.display().to_string(),
    };

    if let Err(error) = app.emit(EVENT_DONE, payload) {
        tracing::debug!(%error, "failed to emit scan-done");
    }
}

fn emit_error(app: &AppHandle, job_id: &str, message: String) {
    let payload = ScanError {
        job_id: job_id.to_string(),
        message,
    };

    if let Err(error) = app.emit(EVENT_ERROR, payload) {
        tracing::debug!(%error, "failed to emit scan-error");
    }
}

pub fn user_message(error: &EsclError) -> String {
    match error {
        EsclError::Http { status: 503, .. } => "the device is busy".to_string(),
        EsclError::Http { status: 409, .. } => {
            "the device does not support this combination of settings".to_string()
        }
        EsclError::Http { status: 500, body } if mentions_feeder(body) => {
            "the feeder is out of paper".to_string()
        }
        EsclError::Timeout => "the device is not responding".to_string(),
        EsclError::Transport(_) => "no connection to the device".to_string(),
        EsclError::InvalidParams(message) => message.clone(),
        EsclError::Http { status, .. } => format!("the device returned HTTP {status}"),
        EsclError::Parse(_) => "the device returned an unexpected response".to_string(),
        EsclError::InvalidUrl(_) => "invalid device address".to_string(),
    }
}

fn mentions_feeder(body: &str) -> bool {
    let body = body.to_ascii_lowercase();

    ["feeder", "adf", "no document", "paper"]
        .iter()
        .any(|needle| body.contains(needle))
}

pub use caps::{get_capabilities, get_startup_error, open_scan_folder};
pub use scan::{cancel_scan, start_scan};

pub use caps::{
    __cmd__get_capabilities, __cmd__get_startup_error, __cmd__open_scan_folder,
    __tauri_command_name_get_capabilities, __tauri_command_name_get_startup_error,
    __tauri_command_name_open_scan_folder,
};
pub use scan::{
    __cmd__cancel_scan, __cmd__start_scan, __tauri_command_name_cancel_scan,
    __tauri_command_name_start_scan,
};

#[cfg(test)]
mod tests;
