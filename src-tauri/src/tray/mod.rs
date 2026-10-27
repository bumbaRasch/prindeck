use std::error::Error;
use std::sync::atomic::Ordering;

mod icons;

#[cfg(test)]
mod tests;

use serde::Serialize;
use tauri::{
    App, AppHandle, Emitter, Manager,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};

use crate::commands::{ACTIVITY_IDLE, AppState};
use crate::status;

use icons::{TrayIconState, TrayInput, TrayState};

const MAIN_WINDOW: &str = "main";
const TRAY_ID: &str = "main";

const EVENT_NAVIGATE: &str = "navigate";
const SCAN_SCREEN: &str = "scan";

const MENU_SHOW: &str = "show";
const MENU_SCAN: &str = "scan";
const MENU_PANEL: &str = "panel";
const MENU_QUIT: &str = "quit";

const MENU_SHOW_LABEL: &str = "Показать окно";
const MENU_SCAN_LABEL: &str = "Скан";
const MENU_PANEL_LABEL: &str = "Открыть панель";
const MENU_QUIT_LABEL: &str = "Выход";

#[derive(Debug, Clone, Serialize)]
struct NavigateEvent {
    screen: &'static str,
}

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

fn open_scan_screen(app: &AppHandle) {
    show_main_window(app);

    let payload = NavigateEvent {
        screen: SCAN_SCREEN,
    };
    if let Err(error) = app.emit(EVENT_NAVIGATE, payload) {
        tracing::debug!(%error, "failed to emit the navigate event");
    }
}

fn open_printer_panel(app: &AppHandle) {
    if let Err(error) = crate::commands::open_status_page(app) {
        tracing::warn!(%error, "failed to open the printer panel from the tray");
    }
}

pub fn update(app: &AppHandle, input: TrayInput) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        tracing::debug!("the tray icon is missing for an update");
        return;
    };

    if let Err(error) = tray.set_icon(Some(icons::icon(input.icon_state()))) {
        tracing::warn!(%error, icon = ?input.icon_state(), "failed to update the tray icon");
    }
}

pub fn refresh(app: &AppHandle) {
    let state = TrayState::from_status(&status::cached(app));
    let activity = app
        .try_state::<AppState>()
        .map(|managed| managed.activity.load(Ordering::Relaxed))
        .unwrap_or(ACTIVITY_IDLE);

    update(app, TrayInput { state, activity });
}

pub fn setup(app: &App) -> Result<(), Box<dyn Error>> {
    let show = MenuItem::with_id(app, MENU_SHOW, MENU_SHOW_LABEL, true, None::<&str>)?;
    let scan = MenuItem::with_id(app, MENU_SCAN, MENU_SCAN_LABEL, true, None::<&str>)?;
    let panel = MenuItem::with_id(app, MENU_PANEL, MENU_PANEL_LABEL, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, MENU_QUIT, MENU_QUIT_LABEL, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &scan, &panel, &quit])?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icons::icon(TrayIconState::Idle))
        .tooltip("prindeck")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            MENU_SHOW => show_main_window(app),
            MENU_SCAN => open_scan_screen(app),
            MENU_PANEL => open_printer_panel(app),
            MENU_QUIT => app.exit(0),
            other => tracing::debug!(id = other, "unknown tray menu item"),
        })
        .build(app)?;

    Ok(())
}
