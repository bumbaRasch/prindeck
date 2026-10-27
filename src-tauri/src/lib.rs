pub mod commands;
pub mod config;
pub mod escl;
pub mod history;
pub mod snmp;
pub mod status;
#[cfg(desktop)]
pub mod tray;

use std::sync::Mutex;

use commands::AppState;
use history::{History, HistoryHandle};
use tauri::{Manager, WindowEvent};

fn init_tracing() {
    use tracing_subscriber::EnvFilter;

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("prindeck=info"));

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();

    let builder = tauri::Builder::default();

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
        tray::show_main_window(app);
    }));

    builder
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::initialize())
        .setup(|app| {
            #[cfg(desktop)]
            if let Err(error) = tray::setup(app) {
                tracing::error!(%error, "failed to create the tray icon");
            }

            let host = app.state::<AppState>().config.device.host.clone();
            app.manage(status::StatusCache::new(host));

            let history = match history::default_path().and_then(|path| History::open(&path)) {
                Ok(history) => Some(history),
                Err(error) => {
                    tracing::error!(%error, "failed to open the history database");
                    None
                }
            };
            app.manage(HistoryHandle(Mutex::new(history)));

            status::start_polling(app.handle().clone());

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if let Err(error) = window.hide() {
                    tracing::error!(%error, "failed to hide the window");
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_capabilities,
            commands::start_scan,
            commands::cancel_scan,
            commands::open_scan_folder,
            commands::get_startup_error,
            commands::get_status,
            commands::refresh_status,
            commands::open_printer_panel,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
