use std::path::{Path, PathBuf};

use chrono::Local;
use reqwest::Url;
use tauri::{AppHandle, Manager};

use super::scan_files::write_pages;
use super::scan_pages::{PagesOutcome, fetch_pages};
use super::{AppState, ScanFailure, ScanPhase, emit_done, emit_error, emit_progress, user_message};
use crate::escl::{DocFormat, EsclClient};

#[derive(Debug)]
pub(super) enum TerminalEvent {
    Done {
        files: Vec<PathBuf>,
        partial: bool,
        warning: Option<String>,
    },
    Canceled,
    Error {
        failure: ScanFailure,
    },
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

    let outcome = fetch_pages(&client, &job, format, |pages| {
        emit_progress(&app, &job_id, ScanPhase::Scanning, pages);
    })
    .await;

    let terminal = finish_scan(&client, &scan_dir, &job_id, &job, format, outcome).await;

    match terminal {
        TerminalEvent::Done {
            files,
            partial,
            warning,
        } => {
            tracing::info!(%job_id, files = ?files, partial, "scan saved");
            emit_done(&app, &job_id, &files, partial, warning);
        }
        TerminalEvent::Canceled => {
            tracing::info!(%job_id, "scan canceled");
            emit_progress(&app, &job_id, ScanPhase::Canceled, 0);
        }
        TerminalEvent::Error { failure } => {
            tracing::warn!(%job_id, kind = ?failure.kind, message = %failure.message, "scan failed");
            emit_error(&app, &job_id, failure);
        }
    }

    forget_job(&app, &job_id);
}

pub(super) async fn finish_scan(
    client: &EsclClient,
    scan_dir: &Path,
    job_id: &str,
    job: &Url,
    format: DocFormat,
    outcome: PagesOutcome,
) -> TerminalEvent {
    let partial = !matches!(outcome, PagesOutcome::Complete { .. });

    if partial {
        best_effort_cancel(client, job).await;
    }

    let pages = outcome_pages(&outcome);
    let (files, storage) = write_pages(scan_dir, job_id, format, pages, partial, Local::now());
    let terminal = terminal_for(&outcome, &files, storage);

    if let TerminalEvent::Done {
        warning: Some(message),
        ..
    } = &terminal
    {
        tracing::warn!(%job_id, %message, "scan saved with a warning");
    }

    terminal
}

pub(super) fn terminal_for(
    outcome: &PagesOutcome,
    files: &[PathBuf],
    storage: Option<ScanFailure>,
) -> TerminalEvent {
    if let Some(failure) = storage {
        return if files.is_empty() {
            TerminalEvent::Error { failure }
        } else {
            TerminalEvent::Done {
                files: files.to_vec(),
                partial: true,
                warning: Some(failure.message),
            }
        };
    }

    match outcome {
        PagesOutcome::Complete { .. } => TerminalEvent::Done {
            files: files.to_vec(),
            partial: false,
            warning: None,
        },
        PagesOutcome::Canceled { pages } if !pages.is_empty() => TerminalEvent::Done {
            files: files.to_vec(),
            partial: true,
            warning: None,
        },
        PagesOutcome::Canceled { .. } => TerminalEvent::Canceled,
        PagesOutcome::Failed { pages, error } if !pages.is_empty() => TerminalEvent::Done {
            files: files.to_vec(),
            partial: true,
            warning: Some(user_message(error)),
        },
        PagesOutcome::Failed { error, .. } => TerminalEvent::Error {
            failure: ScanFailure::from_escl(error),
        },
    }
}

fn outcome_pages(outcome: &PagesOutcome) -> &[Vec<u8>] {
    match outcome {
        PagesOutcome::Complete { pages } | PagesOutcome::Canceled { pages } => pages,
        PagesOutcome::Failed { pages, .. } => pages,
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

    let mut jobs = state
        .jobs
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    jobs.remove(job_id);
}
