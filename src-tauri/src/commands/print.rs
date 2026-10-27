use std::io;
use std::path::PathBuf;

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

const PRINT_COPIES_RANGE_LABEL: &str = "число копий — от 1 до 99";
const PRINT_SIDES_UNKNOWN_LABEL: &str = "неизвестный режим двусторонней печати";
const PRINT_JOB_OUTSIDE_QUEUE_LABEL: &str = "задание не относится к настроенной очереди";
const PRINT_QUEUE_MISSING_LABEL: &str = "очередь CUPS не найдена";
const PRINT_CUPS_MISSING_LABEL: &str = "CUPS на этом компьютере недоступна";
const PRINT_SEND_FAILED_LABEL: &str = "не удалось отправить задание";
const PRINT_QUEUE_FAILED_LABEL: &str = "не удалось получить очередь";
const PRINT_CANCEL_FAILED_LABEL: &str = "не удалось отменить задание";

fn sides_from_ipc(value: &str) -> Result<Sides, String> {
    match value {
        SIDES_ONE => Ok(Sides::OneSided),
        SIDES_LONG => Ok(Sides::LongEdge),
        SIDES_SHORT => Ok(Sides::ShortEdge),
        _ => Err(PRINT_SIDES_UNKNOWN_LABEL.to_string()),
    }
}

fn check_copies(copies: u32) -> Result<(), String> {
    if (MIN_COPIES..=MAX_COPIES).contains(&copies) {
        Ok(())
    } else {
        Err(PRINT_COPIES_RANGE_LABEL.to_string())
    }
}

fn job_belongs_to_queue(job_id: &str, queue: &str) -> bool {
    job_id.starts_with(&format!("{queue}-"))
}

fn print_message(error: &PrintError) -> String {
    match error {
        PrintError::Io { source, .. } if source.kind() == io::ErrorKind::NotFound => {
            PRINT_CUPS_MISSING_LABEL.to_string()
        }
        PrintError::Failed { stderr, .. } if mentions_missing_queue(stderr) => {
            PRINT_QUEUE_MISSING_LABEL.to_string()
        }
        PrintError::Io { program, .. }
        | PrintError::Timeout { program }
        | PrintError::Failed { program, .. }
        | PrintError::UnexpectedOutput { program, .. } => program_label(program).to_string(),
        PrintError::MissingJobId => PRINT_CANCEL_FAILED_LABEL.to_string(),
    }
}

fn program_label(program: &str) -> &'static str {
    match program {
        "lp" => PRINT_SEND_FAILED_LABEL,
        "lpstat" => PRINT_QUEUE_FAILED_LABEL,
        _ => PRINT_CANCEL_FAILED_LABEL,
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
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|error| format!("the CUPS task did not finish: {error}"))?
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
) -> Result<String, String> {
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
pub async fn get_queue(state: State<'_, AppState>) -> Result<Vec<CupsJob>, String> {
    let queue = state.config.get().cups_queue.clone();
    run_cups(move || cups::queue_jobs(&queue)).await
}

#[tauri::command]
pub async fn cancel_print(state: State<'_, AppState>, job_id: String) -> Result<(), String> {
    if !job_belongs_to_queue(&job_id, &state.config.get().cups_queue) {
        tracing::warn!(
            job_id,
            "refused to cancel a job outside the configured queue"
        );
        return Err(PRINT_JOB_OUTSIDE_QUEUE_LABEL.to_string());
    }

    run_cups(move || cups::cancel(&job_id)).await
}
