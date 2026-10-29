use std::io;
use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, State};

use super::AppState;
use super::activity::{self, ACTIVITY_PRINTING};
use crate::print::{self as cups, CupsJob, PrintError, PrintOptions, Sides};

#[cfg(test)]
mod tests;

const SIDES_ONE: &str = "one-sided";
const SIDES_LONG: &str = "two-sided-long-edge";
const SIDES_SHORT: &str = "two-sided-short-edge";
const MIN_COPIES: u32 = 1;
const MAX_COPIES: u32 = 99;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum PrintIpcError {
    CopiesRange,
    SidesUnknown,
    JobOutsideQueue,
    CupsMissing,
    QueueMissing,
    SendFailed,
    QueueFailed,
    CancelFailed,
}

fn sides_from_ipc(value: &str) -> Result<Sides, PrintIpcError> {
    match value {
        SIDES_ONE => Ok(Sides::OneSided),
        SIDES_LONG => Ok(Sides::LongEdge),
        SIDES_SHORT => Ok(Sides::ShortEdge),
        _ => Err(PrintIpcError::SidesUnknown),
    }
}

fn check_copies(copies: u32) -> Result<(), PrintIpcError> {
    if (MIN_COPIES..=MAX_COPIES).contains(&copies) {
        Ok(())
    } else {
        Err(PrintIpcError::CopiesRange)
    }
}

fn job_belongs_to_queue(job_id: &str, queue: &str) -> bool {
    job_id.starts_with(&format!("{queue}-"))
}

fn cancel_job_id(job_id: &str, queue: &str) -> Result<String, PrintIpcError> {
    if job_belongs_to_queue(job_id, queue) {
        Ok(job_id.to_string())
    } else {
        Err(PrintIpcError::JobOutsideQueue)
    }
}

fn print_message(error: &PrintError) -> PrintIpcError {
    match error {
        PrintError::Io { source, .. } if source.kind() == io::ErrorKind::NotFound => {
            PrintIpcError::CupsMissing
        }
        PrintError::Failed { stderr, .. } if mentions_missing_queue(stderr) => {
            PrintIpcError::QueueMissing
        }
        PrintError::Io { program, .. }
        | PrintError::Timeout { program }
        | PrintError::Failed { program, .. }
        | PrintError::UnexpectedOutput { program, .. } => program_label(program),
        PrintError::MissingJobId => PrintIpcError::CancelFailed,
    }
}

fn program_label(program: &str) -> PrintIpcError {
    match program {
        "lp" => PrintIpcError::SendFailed,
        "lpstat" => PrintIpcError::QueueFailed,
        _ => PrintIpcError::CancelFailed,
    }
}

fn mentions_missing_queue(stderr: &str) -> bool {
    let stderr = stderr.to_ascii_lowercase();

    ["invalid destination", "unknown destination", "not valid"]
        .iter()
        .any(|needle| stderr.contains(needle))
}

async fn run_cups<T: Send + 'static>(
    task: impl FnOnce() -> Result<T, PrintError> + Send + 'static,
) -> Result<T, PrintIpcError> {
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|error| {
            tracing::warn!(%error, "the CUPS task did not finish");
            PrintIpcError::CancelFailed
        })?
        .map_err(|error| {
            tracing::warn!(%error, "CUPS command failed");
            print_message(&error)
        })
}

#[tauri::command]
pub async fn print_file(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    copies: u32,
    sides: String,
) -> Result<String, PrintIpcError> {
    let sides = sides_from_ipc(&sides)?;
    check_copies(copies)?;

    let queue = state.config.get().cups_queue.clone();
    let path = PathBuf::from(path);
    let options = PrintOptions { copies, sides };

    activity::set(&app, ACTIVITY_PRINTING);
    let submitted = run_cups(move || cups::print_file(&queue, &path, &options)).await;
    activity::clear(&app, ACTIVITY_PRINTING);
    submitted
}

#[tauri::command]
pub async fn get_queue(state: State<'_, AppState>) -> Result<Vec<CupsJob>, PrintIpcError> {
    let queue = state.config.get().cups_queue.clone();
    run_cups(move || cups::queue_jobs(&queue)).await
}

#[tauri::command]
pub async fn cancel_print(state: State<'_, AppState>, job_id: String) -> Result<(), PrintIpcError> {
    let job_id = match cancel_job_id(&job_id, &state.config.get().cups_queue) {
        Ok(job_id) => job_id,
        Err(error) => {
            tracing::warn!(
                job_id,
                "refused to cancel a job outside the configured queue"
            );
            return Err(error);
        }
    };

    run_cups(move || cups::cancel(&job_id)).await
}
