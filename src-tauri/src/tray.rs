use std::error::Error;

use tauri::{
    App, AppHandle, Manager,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};

const MAIN_WINDOW: &str = "main";

const MENU_SHOW: &str = "show";
const MENU_QUIT: &str = "quit";

pub fn show_main_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
        tracing::error!(window = MAIN_WINDOW, "main window not found");
        return;
    };

    if let Err(error) = window.unminimize() {
        tracing::debug!(%error, "failed to unminimize the window");
    }
    if let Err(error) = window.show() {
        tracing::error!(%error, "failed to show the window");
    }
    if let Err(error) = window.set_focus() {
        tracing::error!(%error, "failed to focus the window");
    }
}

pub fn setup(app: &App) -> Result<(), Box<dyn Error>> {
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or("the app icon is missing from the configuration")?;

    let show = MenuItem::with_id(app, MENU_SHOW, "Показать окно", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, MENU_QUIT, "Выход", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("prindeck")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            MENU_SHOW => show_main_window(app),
            MENU_QUIT => app.exit(0),
            other => tracing::debug!(id = other, "unknown tray menu item"),
        })
        .build(app)?;

    Ok(())
}
