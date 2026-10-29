use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::scan_errors::{ScanErrorKind, ScanFailure};

pub const EVENT_PROGRESS: &str = "scan-progress";
pub const EVENT_DONE: &str = "scan-done";
pub const EVENT_ERROR: &str = "scan-error";

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
    pub warning: Option<ScanFailure>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanError {
    pub job_id: String,
    pub kind: ScanErrorKind,
    pub message: String,
}

pub fn emit_progress(app: &AppHandle, job_id: &str, phase: ScanPhase, pages: u32) {
    let payload = ScanProgress {
        job_id: job_id.to_string(),
        phase,
        pages,
    };

    if let Err(error) = app.emit(EVENT_PROGRESS, payload) {
        tracing::debug!(%error, "failed to emit scan-progress");
    }
}

pub fn emit_done(
    app: &AppHandle,
    job_id: &str,
    files: &[PathBuf],
    partial: bool,
    warning: Option<ScanFailure>,
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

pub fn emit_error(app: &AppHandle, job_id: &str, failure: ScanFailure) {
    let payload = ScanError {
        job_id: job_id.to_string(),
        kind: failure.kind,
        message: failure.message,
    };

    if let Err(error) = app.emit(EVENT_ERROR, payload) {
        tracing::debug!(%error, "failed to emit scan-error");
    }
}
