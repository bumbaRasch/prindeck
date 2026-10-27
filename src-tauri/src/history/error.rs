use std::fmt;
use std::io;

use rusqlite::Error as SqliteError;

#[derive(Debug)]
pub enum HistoryError {
    Io(io::Error),
    Sqlite(SqliteError),
    UnsupportedSchema(i32),
    NoDataDir,
}

impl fmt::Display for HistoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HistoryError::Io(error) => write!(f, "history database I/O error: {error}"),
            HistoryError::Sqlite(error) => write!(f, "history database error: {error}"),
            HistoryError::UnsupportedSchema(version) => write!(
                f,
                "history database schema version {version} is newer than supported"
            ),
            HistoryError::NoDataDir => write!(f, "failed to determine the data directory"),
        }
    }
}

impl std::error::Error for HistoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HistoryError::Io(error) => Some(error),
            HistoryError::Sqlite(error) => Some(error),
            _ => None,
        }
    }
}
