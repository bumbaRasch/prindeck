use tauri::{AppHandle, State};

use super::caps::load_caps;
use super::scan_jobs::{best_effort_cancel, run_scan};
use super::{AppState, ScanErrorKind, ScanFailure, ScanPhase, emit_progress, lock, user_message};
use crate::escl::{CancelOutcome, ScanParams, ScannerState, fetch_timeout};

pub const JOB_PATH_PREFIX: &str = "/eSCL/ScanJobs/";

const INVALID_JOB_ID: &str = "invalid job id";
const JOB_NOT_FOUND: &str = "job not found";

pub enum StartRejection {
    Busy,
    DuplicateJob,
}

pub fn rejection_failure(rejection: StartRejection) -> ScanFailure {
    match rejection {
        StartRejection::Busy => ScanFailure::new(ScanErrorKind::Busy, "the device is busy"),
        StartRejection::DuplicateJob => {
            ScanFailure::new(ScanErrorKind::Invalid, "this job is already running")
        }
    }
}

pub fn validate_job_id(job_id: &str) -> Result<(), String> {
    let Some(rest) = job_id.strip_prefix(JOB_PATH_PREFIX) else {
        return Err(INVALID_JOB_ID.to_string());
    };

    let valid = !rest.is_empty()
        && rest.len() <= 128
        && rest
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_');

    if valid {
        Ok(())
    } else {
        Err(INVALID_JOB_ID.to_string())
    }
}

#[tauri::command]
pub async fn start_scan(
    app: AppHandle,
    state: State<'_, AppState>,
    params: ScanParams,
) -> Result<String, ScanFailure> {
    let caps = load_caps(&state.client(), &state.caps)
        .await
        .map_err(|error| {
            tracing::warn!(%error, "capabilities request failed");
            ScanFailure::from_escl(&error)
        })?;
    let document_timeout = fetch_timeout(params.source, &caps);

    let status = state.client().status().await.map_err(|error| {
        tracing::warn!(%error, "scanner status request failed");
        ScanFailure::from_escl(&error)
    })?;

    if status.state != ScannerState::Idle {
        return Err(rejection_failure(StartRejection::Busy));
    }

    let job = state.client().start(&params).await.map_err(|error| {
        tracing::warn!(%error, "scan start request failed");
        ScanFailure::from_escl(&error)
    })?;
    let job_id = job.path().to_string();

    let duplicate = {
        let mut jobs = lock(&state.jobs)
            .map_err(|message| ScanFailure::new(ScanErrorKind::Device, message))?;
        let duplicate = jobs.contains_key(&job_id);
        if !duplicate {
            jobs.insert(job_id.clone(), job.clone());
        }
        duplicate
    };

    if duplicate {
        best_effort_cancel(&state.client(), &job).await;
        return Err(rejection_failure(StartRejection::DuplicateJob));
    }

    tracing::info!(
        %job_id,
        source = ?params.source,
        dpi = params.dpi,
        color = ?params.color,
        format = ?params.format,
        "scan started"
    );

    emit_progress(&app, &job_id, ScanPhase::Created, 0);

    let client = state.client();
    let scan_dir = state.config.get().scan_dir.clone();
    let format = params.format;
    let task_job_id = job_id.clone();

    tauri::async_runtime::spawn(run_scan(
        app,
        client,
        scan_dir,
        task_job_id,
        job,
        format,
        document_timeout,
    ));

    Ok(job_id)
}

#[tauri::command]
pub async fn cancel_scan(state: State<'_, AppState>, job_id: String) -> Result<(), String> {
    let job = {
        validate_job_id(&job_id)?;
        let jobs = lock(&state.jobs)?;
        jobs.get(&job_id)
            .cloned()
            .ok_or_else(|| JOB_NOT_FOUND.to_string())?
    };

    match state.client().cancel(&job).await {
        Ok(CancelOutcome::Canceled | CancelOutcome::AlreadyFinished) => Ok(()),
        Err(error) => {
            tracing::warn!(%error, %job_id, "scan cancel request failed");
            Err(user_message(&error))
        }
    }
}
