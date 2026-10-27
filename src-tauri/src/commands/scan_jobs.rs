use std::fs;
use std::path::{Path, PathBuf};

use chrono::Local;
use reqwest::Url;
use tauri::{AppHandle, Manager};

use super::scan_files::{format_matches, part_path, remove_part, unique_scan_path};
use super::{AppState, ScanPhase, emit_done, emit_error, emit_progress, user_message};
use crate::escl::{DocFormat, EsclClient, NextDoc};

const ADF_NOT_SUPPORTED: &str =
    "the device returned more than one page: the feeder is not supported in this version";
const UNEXPECTED_FORMAT: &str = "the device returned an unexpected format";
const NO_DOCUMENT: &str = "the device returned no document";

enum ScanOutcome {
    Done(PathBuf),
    Canceled,
    Failed(String),
}

pub(super) async fn run_scan(
    app: AppHandle,
    client: EsclClient,
    scan_dir: PathBuf,
    job_id: String,
    job: Url,
    format: DocFormat,
) {
    emit_progress(&app, &job_id, ScanPhase::Scanning, 0);

    let outcome = fetch_document(&client, &scan_dir, &job, &job_id, format).await;

    match outcome {
        ScanOutcome::Done(file) => {
            tracing::info!(%job_id, path = %file.display(), "scan saved");
            emit_done(&app, &job_id, &file);
            emit_progress(&app, &job_id, ScanPhase::Done, 1);
        }
        ScanOutcome::Canceled => {
            tracing::info!(%job_id, "scan canceled");
            emit_progress(&app, &job_id, ScanPhase::Canceled, 0);
        }
        ScanOutcome::Failed(message) => {
            tracing::warn!(%job_id, %message, "scan failed");
            emit_error(&app, &job_id, message);
        }
    }

    forget_job(&app, &job_id);
}

async fn fetch_document(
    client: &EsclClient,
    scan_dir: &Path,
    job: &Url,
    job_id: &str,
    format: DocFormat,
) -> ScanOutcome {
    let part = part_path(scan_dir, job_id);
    let mut written = false;

    loop {
        match client.next_document(job).await {
            Ok(NextDoc::Document(bytes)) => {
                if written {
                    best_effort_cancel(client, job).await;
                    remove_part(&part);
                    return ScanOutcome::Failed(ADF_NOT_SUPPORTED.to_string());
                }

                if !format_matches(format, &bytes) {
                    best_effort_cancel(client, job).await;
                    remove_part(&part);
                    return ScanOutcome::Failed(UNEXPECTED_FORMAT.to_string());
                }

                if let Err(error) = fs::write(&part, &bytes) {
                    remove_part(&part);
                    best_effort_cancel(client, job).await;
                    return ScanOutcome::Failed(format!("failed to save the file: {error}"));
                }

                written = true;
            }
            Ok(NextDoc::End) => {
                if !written {
                    remove_part(&part);
                    return ScanOutcome::Failed(NO_DOCUMENT.to_string());
                }

                let target = unique_scan_path(scan_dir, Local::now(), format);
                if let Err(error) = fs::rename(&part, &target) {
                    remove_part(&part);
                    return ScanOutcome::Failed(format!("failed to save the file: {error}"));
                }

                return ScanOutcome::Done(target);
            }
            Ok(NextDoc::Canceled) => {
                remove_part(&part);
                return ScanOutcome::Canceled;
            }
            Err(error) => {
                remove_part(&part);
                best_effort_cancel(client, job).await;
                return ScanOutcome::Failed(user_message(&error));
            }
        }
    }
}

pub(super) async fn best_effort_cancel(client: &EsclClient, job: &Url) {
    if let Err(error) = client.cancel(job).await {
        tracing::debug!(%error, "failed to cancel the job after an error");
    }
}

fn forget_job(app: &AppHandle, job_id: &str) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };

    match state.jobs.lock() {
        Ok(mut jobs) => {
            jobs.remove(job_id);
        }
        Err(error) => tracing::error!(%error, "failed to update the job registry"),
    }
}
