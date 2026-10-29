use std::sync::{Mutex, MutexGuard};

use reqwest::Url;
use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use super::{AppState, FALLBACK_BASE_URL, ScanFailure};
use crate::config::{Config, ConfigError, ConfigField};
use crate::escl::{EsclClient, EsclError, ScanCaps};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StartupKind {
    LoadFailed,
    Invalid,
    Fallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupProblem {
    pub kind: StartupKind,
    pub field: Option<ConfigField>,
    pub detail: String,
}

impl StartupProblem {
    pub(super) fn load(detail: String) -> StartupProblem {
        StartupProblem {
            kind: StartupKind::LoadFailed,
            field: None,
            detail,
        }
    }

    fn config(error: &ConfigError, detail: String) -> StartupProblem {
        match error.field_error() {
            Some(error) => StartupProblem {
                kind: StartupKind::Invalid,
                field: error.field,
                detail,
            },
            None => StartupProblem::load(detail),
        }
    }
}

pub(super) async fn load_caps(
    client: &EsclClient,
    cache: &Mutex<Option<ScanCaps>>,
) -> Result<ScanCaps, EsclError> {
    {
        let cached = caps_guard(cache);
        if let Some(caps) = cached.as_ref() {
            return Ok(caps.clone());
        }
    }

    let caps = client.capabilities().await?;

    {
        let mut cached = caps_guard(cache);
        *cached = Some(caps.clone());
    }

    Ok(caps)
}

fn caps_guard(cache: &Mutex<Option<ScanCaps>>) -> MutexGuard<'_, Option<ScanCaps>> {
    cache
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[tauri::command]
pub async fn get_capabilities(state: State<'_, AppState>) -> Result<ScanCaps, ScanFailure> {
    let client = state.client();
    load_caps(&client, &state.caps)
        .await
        .map_err(|error| ScanFailure::from_escl(&error))
}

#[tauri::command]
pub fn open_scan_folder(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let config = state.config.get();
    config
        .ensure_scan_dir()
        .map_err(|error| error.to_string())?;

    let path = config.scan_dir.display().to_string();

    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|error| format!("failed to open the scan folder: {error}"))
}

#[tauri::command]
pub fn get_startup_error(state: State<'_, AppState>) -> Option<Vec<StartupProblem>> {
    state.startup_error.clone()
}

pub(super) fn resolve_state() -> (Config, EsclClient, Vec<StartupProblem>) {
    resolve_loaded(load_config())
}

fn load_config() -> Result<Config, StartupProblem> {
    let path = Config::default_path().map_err(|error| {
        StartupProblem::load(format!("failed to resolve the config path: {error}"))
    })?;
    Config::load_or_create(&path).map_err(|error| {
        let detail = format!("failed to load the config {}: {error}", path.display());
        StartupProblem::config(&error, detail)
    })
}

pub(super) fn resolve_loaded(
    loaded: Result<Config, StartupProblem>,
) -> (Config, EsclClient, Vec<StartupProblem>) {
    let mut problems = Vec::new();

    match loaded {
        Ok(config) => match build_client(&config) {
            Ok(client) => return (config, client, problems),
            Err(problem) => problems.push(problem),
        },
        Err(problem) => problems.push(problem),
    }

    let fallback = fallback_config();
    problems.push(StartupProblem {
        kind: StartupKind::Fallback,
        field: None,
        detail: "default settings are in use".to_string(),
    });

    match build_client(&fallback) {
        Ok(client) => (fallback, client, problems),
        Err(problem) => {
            problems.push(problem);
            (fallback, fallback_client(), problems)
        }
    }
}

fn fallback_config() -> Config {
    let home = dirs::home_dir().unwrap_or_else(std::env::temp_dir);
    let documents = dirs::document_dir();

    Config::defaults_for(&home, documents.as_deref())
}

fn build_client(config: &Config) -> Result<EsclClient, StartupProblem> {
    let url = config
        .base_url()
        .map_err(|error| StartupProblem::config(&error, error.to_string()))?;
    EsclClient::new(url).map_err(|error| StartupProblem::load(error.to_string()))
}

pub(super) fn rebuild_client(state: &AppState, config: &Config) -> Result<EsclClient, String> {
    let client = build_client(config).map_err(|problem| problem.detail)?;

    *state
        .client
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = client.clone();
    *caps_guard(&state.caps) = None;

    Ok(client)
}

fn fallback_client() -> EsclClient {
    let url = Url::parse(FALLBACK_BASE_URL).expect("fallback base URL is a valid literal");
    EsclClient::new(url).expect("HTTP client without TLS cannot fail to build")
}
