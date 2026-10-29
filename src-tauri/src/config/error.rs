use std::fmt;
use std::io;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ConfigField {
    Host,
    PollSeconds,
    ScanDir,
    NotifyTonerPercent,
    NotifyDrumPercent,
    TonerYieldPages,
    CupsQueue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FieldErrorKind {
    Required,
    Format,
    Range,
    Io,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldError {
    pub field: Option<ConfigField>,
    pub kind: FieldErrorKind,
}

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Serde(serde_json::Error),
    InvalidHost(String),
    InvalidCupsQueue(String),
    InvalidScanDir(String),
    InvalidNotify(String),
    InvalidToner(String),
    NoConfigDir,
    NoHomeDir,
}

impl ConfigError {
    pub fn field_error(&self) -> Option<FieldError> {
        let field = match self {
            ConfigError::InvalidHost(_) => ConfigField::Host,
            ConfigError::InvalidCupsQueue(_) => ConfigField::CupsQueue,
            ConfigError::InvalidScanDir(_) => ConfigField::ScanDir,
            ConfigError::InvalidNotify(message) if message.contains("drumPercent") => {
                ConfigField::NotifyDrumPercent
            }
            ConfigError::InvalidNotify(_) => ConfigField::NotifyTonerPercent,
            ConfigError::InvalidToner(_) => ConfigField::TonerYieldPages,
            ConfigError::Io(_)
            | ConfigError::Serde(_)
            | ConfigError::NoConfigDir
            | ConfigError::NoHomeDir => return None,
        };
        let kind = if matches!(
            self,
            ConfigError::InvalidNotify(_) | ConfigError::InvalidToner(_)
        ) {
            FieldErrorKind::Range
        } else {
            FieldErrorKind::Format
        };

        Some(FieldError {
            field: Some(field),
            kind,
        })
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(error) => write!(f, "I/O error: {error}"),
            ConfigError::Serde(error) => write!(f, "config parse error: {error}"),
            ConfigError::InvalidHost(message) => {
                write!(f, "invalid device address: {message}")
            }
            ConfigError::InvalidCupsQueue(message) => {
                write!(f, "invalid CUPS queue: {message}")
            }
            ConfigError::InvalidScanDir(message) => {
                write!(f, "invalid scan folder: {message}")
            }
            ConfigError::InvalidNotify(message) => {
                write!(f, "invalid notification settings: {message}")
            }
            ConfigError::InvalidToner(message) => {
                write!(f, "invalid toner settings: {message}")
            }
            ConfigError::NoConfigDir => {
                write!(f, "failed to determine the config directory")
            }
            ConfigError::NoHomeDir => write!(f, "failed to determine the home directory"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io(error) => Some(error),
            ConfigError::Serde(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(error: io::Error) -> Self {
        ConfigError::Io(error)
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(error: serde_json::Error) -> Self {
        ConfigError::Serde(error)
    }
}

#[cfg(test)]
mod tests;
