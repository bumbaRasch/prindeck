use tauri::{AppHandle, Manager, State};

use super::AppState;
use super::caps::rebuild_client;
#[cfg(desktop)]
use crate::autostart;
use crate::config::{Config, ConfigError, FieldError, FieldErrorKind};
use crate::notify;

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Config {
    state.config.get()
}

#[tauri::command]
pub fn set_config(
    app: AppHandle,
    state: State<'_, AppState>,
    config: Config,
) -> Result<Config, FieldError> {
    config.validate().map_err(field_error)?;

    let path = Config::default_path().map_err(field_error)?;
    config.save(&path).map_err(field_error)?;

    let previous = state.config.get();
    state.config.set(config);

    apply_live(&app, &state, &previous).map_err(live_field_error)?;
    Ok(state.config.get())
}

pub(super) fn field_error(error: ConfigError) -> FieldError {
    let mapped = error.field_error().unwrap_or(FieldError {
        field: None,
        kind: FieldErrorKind::Io,
    });
    tracing::warn!(%error, "config operation failed");
    mapped
}

pub(super) fn live_field_error(error: String) -> FieldError {
    tracing::warn!(%error, "failed to apply the config");
    FieldError {
        field: None,
        kind: FieldErrorKind::Io,
    }
}

fn apply_live(app: &AppHandle, state: &AppState, previous: &Config) -> Result<(), String> {
    let config = state.config.get();

    if config.device.host != previous.device.host {
        rebuild_client(state, &config)?;
    }

    if config.notify != previous.notify {
        notify::rearm(app);
    }

    #[cfg(desktop)]
    if config.autostart != previous.autostart {
        autostart::apply(app, config.autostart);
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
