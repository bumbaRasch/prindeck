use std::collections::HashMap;
use std::sync::atomic::AtomicU8;
use std::sync::{Arc, Mutex, MutexGuard, RwLock};

use reqwest::Url;

use crate::config::{Config, ConfigHandle};
use crate::escl::{EsclClient, ScanCaps};
use caps::{StartupProblem, resolve_state};

mod activity;
mod caps;
mod events;
mod history;
mod locale;
mod print;
mod scan;
mod scan_errors;
mod scan_files;
mod scan_jobs;
mod scan_pages;
mod settings;
mod status;
mod tray;

const FALLBACK_BASE_URL: &str = "http://127.0.0.1:9/eSCL/";

#[derive(Debug)]
pub struct AppState {
    pub config: ConfigHandle,
    pub activity: Arc<AtomicU8>,
    client: RwLock<EsclClient>,
    caps: Mutex<Option<ScanCaps>>,
    jobs: Mutex<HashMap<String, Url>>,
    startup_error: Option<Vec<StartupProblem>>,
}

impl AppState {
    pub fn initialize() -> AppState {
        let (config, client, problems) = resolve_state();
        let startup_error = (!problems.is_empty()).then_some(problems);
        AppState::from_parts(config, client, startup_error)
    }

    pub(super) fn from_parts(
        config: Config,
        client: EsclClient,
        startup_error: Option<Vec<StartupProblem>>,
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

pub use activity::{ACTIVITY_IDLE, ACTIVITY_PRINTING, ACTIVITY_SCANNING};
pub use caps::{get_capabilities, get_startup_error, open_scan_folder};
pub use events::{
    EVENT_DONE, EVENT_ERROR, EVENT_PROGRESS, ScanDone, ScanError, ScanPhase, ScanProgress,
    emit_done, emit_error, emit_progress,
};
pub use history::get_history;
pub use locale::get_locale_hint;
pub use print::{cancel_print, get_queue, print_file};
pub use scan::{cancel_scan, start_scan};
pub use scan_errors::{ScanErrorKind, ScanFailure, user_message};
pub use settings::{get_config, set_config};
pub use status::{
    get_status, open_firmware_page, open_printer_panel, open_status_page, refresh_status,
};
pub use tray::set_tray_labels;

pub use caps::{
    __cmd__get_capabilities, __cmd__get_startup_error, __cmd__open_scan_folder,
    __tauri_command_name_get_capabilities, __tauri_command_name_get_startup_error,
    __tauri_command_name_open_scan_folder,
};
pub use history::{__cmd__get_history, __tauri_command_name_get_history};
pub use locale::{__cmd__get_locale_hint, __tauri_command_name_get_locale_hint};
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
    __cmd__get_status, __cmd__open_firmware_page, __cmd__open_printer_panel, __cmd__refresh_status,
    __tauri_command_name_get_status, __tauri_command_name_open_firmware_page,
    __tauri_command_name_open_printer_panel, __tauri_command_name_refresh_status,
};
pub use tray::{__cmd__set_tray_labels, __tauri_command_name_set_tray_labels};

#[cfg(test)]
mod tests;
