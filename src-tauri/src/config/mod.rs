use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

mod error;
mod host;

#[cfg(test)]
mod tests;

pub use error::ConfigError;

pub const DEFAULT_HOST: &str = "192.168.187.36";
pub const DEFAULT_CUPS_QUEUE: &str = "Brother_MFC_L2710DW_series";
pub const DEFAULT_SNMP_COMMUNITY: &str = "public";
pub const DEFAULT_POLL_SECONDS: u64 = 300;
pub const MIN_POLL_SECONDS: u64 = 60;
pub const MAX_POLL_SECONDS: u64 = 3600;

fn default_cups_queue() -> String {
    DEFAULT_CUPS_QUEUE.to_string()
}

fn default_snmp() -> SnmpConfig {
    SnmpConfig {
        community: DEFAULT_SNMP_COMMUNITY.to_string(),
    }
}

fn default_poll_seconds() -> u64 {
    DEFAULT_POLL_SECONDS
}

fn default_community() -> String {
    DEFAULT_SNMP_COMMUNITY.to_string()
}

fn deserialize_poll_seconds<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = serde_json::Value::deserialize(deserializer)?;
    let seconds = raw.as_u64();

    match seconds {
        Some(value) => Ok(value),
        None => {
            tracing::warn!(
                value = %raw,
                "pollSeconds must be a whole number of seconds, using the default"
            );
            Ok(DEFAULT_POLL_SECONDS)
        }
    }
}

fn deserialize_community<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = serde_json::Value::deserialize(deserializer)?;

    match raw {
        serde_json::Value::String(text) => Ok(text),
        other => {
            tracing::warn!(
                value = %other,
                "snmp.community must be a string, using the default"
            );
            Ok(DEFAULT_SNMP_COMMUNITY.to_string())
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub device: DeviceConfig,
    pub scan_dir: PathBuf,
    #[serde(default = "default_cups_queue")]
    pub cups_queue: String,
    #[serde(default = "default_snmp")]
    pub snmp: SnmpConfig,
    #[serde(
        default = "default_poll_seconds",
        deserialize_with = "deserialize_poll_seconds"
    )]
    pub poll_seconds: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceConfig {
    pub host: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SnmpConfig {
    #[serde(
        default = "default_community",
        deserialize_with = "deserialize_community"
    )]
    pub community: String,
}

impl Config {
    pub fn default_path() -> Result<PathBuf, ConfigError> {
        let dir = dirs::config_dir().ok_or(ConfigError::NoConfigDir)?;
        Ok(dir.join("prindeck").join("config.json"))
    }

    pub fn defaults_for(home: &Path, documents: Option<&Path>) -> Config {
        let documents = documents
            .map(Path::to_path_buf)
            .unwrap_or_else(|| home.join("Documents"));

        Config {
            device: DeviceConfig {
                host: DEFAULT_HOST.to_string(),
            },
            scan_dir: documents.join("scans"),
            cups_queue: DEFAULT_CUPS_QUEUE.to_string(),
            snmp: default_snmp(),
            poll_seconds: DEFAULT_POLL_SECONDS,
        }
    }

    pub fn from_json_str(json: &str) -> Result<Config, ConfigError> {
        let config: Config = serde_json::from_str(json).map_err(ConfigError::Serde)?;
        config.validate()?;
        Ok(config)
    }

    pub fn load_or_create(path: &Path) -> Result<Config, ConfigError> {
        match fs::read_to_string(path) {
            Ok(text) => {
                let config = Config::from_json_str(&text)?;
                config.ensure_scan_dir()?;
                Ok(config)
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let config = default_config()?;
                config.validate()?;
                config.ensure_scan_dir()?;

                if let Some(parent) = path.parent()
                    && !parent.as_os_str().is_empty()
                {
                    fs::create_dir_all(parent).map_err(ConfigError::Io)?;
                }

                let json = serde_json::to_string_pretty(&config).map_err(ConfigError::Serde)?;
                fs::write(path, json).map_err(ConfigError::Io)?;
                Ok(config)
            }
            Err(error) => Err(ConfigError::Io(error)),
        }
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        host::validate_host(&self.device.host)?;

        if self.cups_queue.trim().is_empty() {
            return Err(ConfigError::InvalidCupsQueue(
                "the CUPS queue name is not set".to_string(),
            ));
        }

        if self.scan_dir.as_os_str().is_empty() {
            return Err(ConfigError::InvalidScanDir(
                "the scan folder path is not set".to_string(),
            ));
        }

        self.base_url()?;
        Ok(())
    }

    pub fn ensure_scan_dir(&self) -> Result<(), ConfigError> {
        if self.scan_dir.as_os_str().is_empty() {
            return Err(ConfigError::InvalidScanDir(
                "the scan folder path is not set".to_string(),
            ));
        }

        fs::create_dir_all(&self.scan_dir).map_err(ConfigError::Io)
    }

    pub fn poll_interval(&self) -> Duration {
        let clamped = self.poll_seconds.clamp(MIN_POLL_SECONDS, MAX_POLL_SECONDS);
        if clamped != self.poll_seconds {
            tracing::warn!(value = self.poll_seconds, clamped, "poll interval clamped");
        }
        Duration::from_secs(clamped)
    }

    pub fn snmp_community(&self) -> &str {
        self.snmp.community.trim()
    }
}

fn default_config() -> Result<Config, ConfigError> {
    let home = dirs::home_dir().ok_or(ConfigError::NoHomeDir)?;
    let documents = dirs::document_dir();

    Ok(Config::defaults_for(&home, documents.as_deref()))
}
