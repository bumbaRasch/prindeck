use tauri::State;

use crate::history::{HistoryHandle, Snapshot};

const DEFAULT_HISTORY_DAYS: u32 = 90;

#[tauri::command]
pub fn get_history(
    history: State<'_, HistoryHandle>,
    days: Option<u32>,
) -> Result<Vec<Snapshot>, String> {
    let history = history
        .0
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let Some(history) = history.as_ref() else {
        return Err("history is unavailable".to_string());
    };

    history
        .read_days(days.unwrap_or(DEFAULT_HISTORY_DAYS))
        .map_err(|error| error.to_string())
}
