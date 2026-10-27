use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Serde(serde_json::Error),
    InvalidHost(String),
    InvalidCupsQueue(String),
    InvalidScanDir(String),
    NoConfigDir,
    NoHomeDir,
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
