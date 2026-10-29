use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use tauri::{AppHandle, Manager};

use super::AppState;

pub const ACTIVITY_IDLE: u8 = 0;
pub const ACTIVITY_SCANNING: u8 = 1;
pub const ACTIVITY_PRINTING: u8 = 2;

pub fn set(app: &AppHandle, activity: u8) {
    if let Some(flag) = flag(app) {
        flag.store(activity, Ordering::Relaxed);
    }
    refresh_tray(app);
}

pub fn clear(app: &AppHandle, finished: u8) {
    if let Some(flag) = flag(app) {
        let still_own = flag.compare_exchange(
            finished,
            ACTIVITY_IDLE,
            Ordering::Relaxed,
            Ordering::Relaxed,
        );
        if still_own.is_err() {
            tracing::debug!("kept the activity flag owned by a newer run");
        }
    }
    refresh_tray(app);
}

fn flag(app: &AppHandle) -> Option<Arc<AtomicU8>> {
    app.try_state::<AppState>()
        .map(|state| state.activity.clone())
}

#[cfg(desktop)]
fn refresh_tray(app: &AppHandle) {
    crate::tray::refresh(app);
}

#[cfg(not(desktop))]
fn refresh_tray(_: &AppHandle) {}
