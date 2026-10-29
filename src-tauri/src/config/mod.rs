use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

mod error;
mod handle;
mod host;
mod notify;
mod store;

#[cfg(test)]
mod tests;

pub use error::{ConfigError, ConfigField, FieldError, FieldErrorKind};
pub use handle::ConfigHandle;
pub use notify::{NotifyConfig, TonerConfig};

pub const DEFAULT_HOST: &str = "192.0.2.10";
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
    #[serde(default)]
    pub autostart: bool,
    pub notify: NotifyConfig,
    pub toner: TonerConfig,
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
            autostart: false,
            notify: NotifyConfig::defaults(),
            toner: TonerConfig::defaults(),
        }
    }

    pub fn from_json_str(json: &str) -> Result<Config, ConfigError> {
        let config: Config = serde_json::from_str(json).map_err(ConfigError::Serde)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        host::validate_host(&self.device.host)?;
        host::validate_queue(&self.cups_queue)?;
        self.notify.validate()?;
        self.toner.validate()?;

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
