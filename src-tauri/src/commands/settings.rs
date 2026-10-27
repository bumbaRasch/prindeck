use tauri::{AppHandle, Manager, State};

use super::AppState;
use super::caps::rebuild_client;
use crate::config::Config;

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Config {
    state.config.get()
}

#[tauri::command]
pub fn set_config(
    app: AppHandle,
    state: State<'_, AppState>,
    config: Config,
) -> Result<Config, String> {
    config.validate().map_err(|error| error.to_string())?;

    let path = Config::default_path().map_err(|error| error.to_string())?;
    config.save(&path).map_err(|error| error.to_string())?;

    let previous = state.config.get();
    state.config.set(config);

    apply_live(&app, &state, &previous)?;
    Ok(state.config.get())
}

fn apply_live(app: &AppHandle, state: &AppState, previous: &Config) -> Result<(), String> {
    let config = state.config.get();

    if config.device.host != previous.device.host {
        rebuild_client(state, &config)?;
    }

    if config.scan_dir != previous.scan_dir
        && let Err(error) = app
            .asset_protocol_scope()
            .allow_directory(&config.scan_dir, false)
    {
        tracing::warn!(
            %error,
            dir = %config.scan_dir.display(),
            "failed to allow the scan directory for the asset protocol"
        );
    }

    Ok(())
}
