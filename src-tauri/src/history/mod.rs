use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::Utc;
use rusqlite::{Connection, params};

use crate::status::DeviceStatus;

mod error;

#[cfg(test)]
mod tests;

pub use error::HistoryError;

const SCHEMA_VERSION: i32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub ts: String,
    pub toner_percent: Option<u32>,
    pub drum_percent: Option<u32>,
    pub pages_total: Option<u32>,
    pub duplex_pages: Option<u32>,
    pub drum_pages_left: Option<u32>,
}

impl Snapshot {
    pub fn from_status(status: &DeviceStatus) -> Snapshot {
        Snapshot {
            ts: status
                .polled_at
                .clone()
                .unwrap_or_else(|| Utc::now().to_rfc3339()),
            toner_percent: status.toner_percent,
            drum_percent: status.drum_percent,
            pages_total: status.pages_total,
            duplex_pages: status.duplex_pages,
            drum_pages_left: status.drum_pages_left,
        }
    }
}

#[derive(Debug)]
pub struct History {
    connection: Connection,
}

impl History {
    pub fn open(path: &Path) -> Result<History, HistoryError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(HistoryError::Io)?;
        }

        let connection = Connection::open(path).map_err(HistoryError::Sqlite)?;
        let version: i32 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(HistoryError::Sqlite)?;

        match version.cmp(&SCHEMA_VERSION) {
            Ordering::Less => History::migrate(&connection)?,
            Ordering::Equal => {}
            Ordering::Greater => return Err(HistoryError::UnsupportedSchema(version)),
        }

        Ok(History { connection })
    }

    fn migrate(connection: &Connection) -> Result<(), HistoryError> {
        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS snapshots (
                    ts TEXT PRIMARY KEY,
                    toner_pct INTEGER NULL,
                    drum_pct INTEGER NULL,
                    pages_total INTEGER NULL,
                    duplex_pages INTEGER NULL,
                    drum_pages_left INTEGER NULL
                )",
                (),
            )
            .map_err(HistoryError::Sqlite)?;

        connection
            .pragma_update(None, "user_version", SCHEMA_VERSION)
            .map_err(HistoryError::Sqlite)
    }

    pub fn record(&self, snapshot: &Snapshot) -> Result<(), HistoryError> {
        self.connection
            .execute(
                "INSERT INTO snapshots
                    (ts, toner_pct, drum_pct, pages_total, duplex_pages, drum_pages_left)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    snapshot.ts,
                    snapshot.toner_percent,
                    snapshot.drum_percent,
                    snapshot.pages_total,
                    snapshot.duplex_pages,
                    snapshot.drum_pages_left
                ],
            )
            .map_err(HistoryError::Sqlite)?;
        Ok(())
    }

    pub fn snapshots(&self) -> Result<Vec<Snapshot>, HistoryError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT ts, toner_pct, drum_pct, pages_total, duplex_pages, drum_pages_left
                 FROM snapshots
                 ORDER BY ts",
            )
            .map_err(HistoryError::Sqlite)?;

        let rows = statement
            .query_map([], |row| {
                Ok(Snapshot {
                    ts: row.get(0)?,
                    toner_percent: row.get(1)?,
                    drum_percent: row.get(2)?,
                    pages_total: row.get(3)?,
                    duplex_pages: row.get(4)?,
                    drum_pages_left: row.get(5)?,
                })
            })
            .map_err(HistoryError::Sqlite)?;

        rows.collect::<Result<Vec<Snapshot>, _>>()
            .map_err(HistoryError::Sqlite)
    }
}

pub fn default_path() -> Result<PathBuf, HistoryError> {
    let dir = dirs::data_dir().ok_or(HistoryError::NoDataDir)?;
    Ok(dir.join("prindeck").join("history.db"))
}

#[derive(Debug)]
pub struct HistoryHandle(pub Mutex<Option<History>>);

pub fn try_record(handle: &HistoryHandle, snapshot: &Snapshot) {
    let history = handle
        .0
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let Some(history) = history.as_ref() else {
        tracing::warn!("history recording is unavailable");
        return;
    };

    if let Err(error) = history.record(snapshot) {
        tracing::warn!(%error, "failed to record the history snapshot");
    }
}
