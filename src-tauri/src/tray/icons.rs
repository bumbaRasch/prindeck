use tauri::image::Image;
use tauri::include_image;

use crate::status::DeviceStatus;

const ICON_IDLE: Image<'static> = include_image!("./icons/tray-idle.png");
const ICON_WORKING: Image<'static> = include_image!("./icons/tray-working.png");
const ICON_ERROR: Image<'static> = include_image!("./icons/tray-error.png");
const ICON_OFFLINE: Image<'static> = include_image!("./icons/tray-offline.png");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayState {
    Ok,
    Error,
    Offline,
}

impl TrayState {
    pub fn from_status(status: &DeviceStatus) -> TrayState {
        if status.ok {
            TrayState::Ok
        } else {
            TrayState::Offline
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayIconState {
    Idle,
    Working,
    Error,
    Offline,
}

#[derive(Debug, Clone, Copy)]
pub struct TrayInput {
    pub state: TrayState,
    pub activity: u8,
}

impl TrayInput {
    pub fn icon_state(&self) -> TrayIconState {
        icon_state(self.state, self.activity)
    }
}

pub fn icon_state(state: TrayState, activity: u8) -> TrayIconState {
    match state {
        TrayState::Offline => TrayIconState::Offline,
        TrayState::Error => TrayIconState::Error,
        TrayState::Ok if activity != 0 => TrayIconState::Working,
        TrayState::Ok => TrayIconState::Idle,
    }
}

pub fn icon(state: TrayIconState) -> Image<'static> {
    match state {
        TrayIconState::Idle => ICON_IDLE.clone(),
        TrayIconState::Working => ICON_WORKING.clone(),
        TrayIconState::Error => ICON_ERROR.clone(),
        TrayIconState::Offline => ICON_OFFLINE.clone(),
    }
}
