mod poll;

#[cfg(test)]
mod tests;

use std::sync::{Mutex, MutexGuard};

use chrono::Utc;
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::snmp::SnmpCounters;

pub const EVENT_STATUS: &str = "status";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceStatus {
    pub ok: bool,
    pub host: String,
    pub toner_percent: Option<u32>,
    pub drum_percent: Option<u32>,
    pub pages_total: Option<u32>,
    pub duplex_pages: Option<u32>,
    pub drum_pages_left: Option<u32>,
    pub polled_at: Option<String>,
    pub error: Option<String>,
}

impl DeviceStatus {
    fn fresh(host: String) -> DeviceStatus {
        DeviceStatus {
            ok: false,
            host,
            toner_percent: None,
            drum_percent: None,
            pages_total: None,
            duplex_pages: None,
            drum_pages_left: None,
            polled_at: None,
            error: None,
        }
    }
}

#[derive(Debug)]
pub struct StatusCache {
    status: Mutex<DeviceStatus>,
}

impl StatusCache {
    pub fn new(host: String) -> StatusCache {
        StatusCache {
            status: Mutex::new(DeviceStatus::fresh(host)),
        }
    }

    pub fn snapshot(&self) -> DeviceStatus {
        self.lock().clone()
    }

    fn apply_success(&self, counters: SnmpCounters) {
        let mut status = self.lock();
        status.ok = true;
        status.toner_percent = counters.toner_percent;
        status.drum_percent = counters.drum_percent;
        status.pages_total = counters.pages_total;
        status.duplex_pages = counters.duplex_pages;
        status.drum_pages_left = counters.drum_pages_left;
        status.polled_at = Some(now_rfc3339());
        status.error = None;
    }

    fn apply_failure(&self, message: String) {
        let mut status = self.lock();
        status.ok = false;
        status.error = Some(message);
    }

    fn lock(&self) -> MutexGuard<'_, DeviceStatus> {
        self.status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

pub fn cached(app: &AppHandle) -> DeviceStatus {
    let cache = app.state::<StatusCache>();
    cache.snapshot()
}

pub use poll::{poll_now, start_polling};
