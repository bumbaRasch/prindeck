use std::sync::{Mutex, MutexGuard};

use reqwest::Url;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use super::{AppState, FALLBACK_BASE_URL, user_message};
use crate::config::Config;
use crate::escl::{EsclClient, EsclError, ScanCaps};

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
pub async fn get_capabilities(state: State<'_, AppState>) -> Result<ScanCaps, String> {
    let client = state.client();
    load_caps(&client, &state.caps)
        .await
        .map_err(|error| user_message(&error))
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
pub fn get_startup_error(state: State<'_, AppState>) -> Option<String> {
    state.startup_error.clone()
}

pub(super) fn resolve_state() -> (Config, EsclClient, Option<String>) {
    let loaded = Config::default_path()
        .map_err(|error| format!("failed to resolve the config path: {error}"))
        .and_then(|path| {
            Config::load_or_create(&path)
                .map_err(|error| format!("failed to load the config {}: {error}", path.display()))
        });

    let mut problems: Vec<String> = Vec::new();

    match loaded {
        Ok(config) => match build_client(&config) {
            Ok(client) => return (config, client, None),
            Err(problem) => problems.push(problem),
        },
        Err(problem) => problems.push(problem),
    }

    let fallback = fallback_config();
    problems.push("default settings are in use".to_string());

    match build_client(&fallback) {
        Ok(client) => (fallback, client, Some(problems.join("; "))),
        Err(problem) => {
            problems.push(problem);
            (fallback, fallback_client(), Some(problems.join("; ")))
        }
    }
}

fn fallback_config() -> Config {
    let home = dirs::home_dir().unwrap_or_else(std::env::temp_dir);
    let documents = dirs::document_dir();

    Config::defaults_for(&home, documents.as_deref())
}

fn build_client(config: &Config) -> Result<EsclClient, String> {
    let url = config.base_url().map_err(|error| error.to_string())?;
    EsclClient::new(url).map_err(|error| error.to_string())
}

pub(super) fn rebuild_client(state: &AppState, config: &Config) -> Result<EsclClient, String> {
    let client = build_client(config)?;

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
