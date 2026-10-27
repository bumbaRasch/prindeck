use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const DEFAULT_HOST: &str = "192.168.187.36";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub device: DeviceConfig,
    pub scan_dir: PathBuf,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceConfig {
    pub host: String,
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

                if let Some(parent) = path.parent() {
                    if !parent.as_os_str().is_empty() {
                        fs::create_dir_all(parent).map_err(ConfigError::Io)?;
                    }
                }

                let json = serde_json::to_string_pretty(&config).map_err(ConfigError::Serde)?;
                fs::write(path, json).map_err(ConfigError::Io)?;
                Ok(config)
            }
            Err(error) => Err(ConfigError::Io(error)),
        }
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        validate_host(&self.device.host)?;

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

    pub fn base_url(&self) -> Result<reqwest::Url, ConfigError> {
        validate_host(&self.device.host)?;

        reqwest::Url::parse(&format!("http://{}/eSCL/", self.device.host.trim()))
            .map_err(|error| ConfigError::InvalidHost(format!("failed to build the URL: {error}")))
    }
}

fn default_config() -> Result<Config, ConfigError> {
    let home = dirs::home_dir().ok_or(ConfigError::NoHomeDir)?;
    let documents = dirs::document_dir();

    Ok(Config::defaults_for(&home, documents.as_deref()))
}

fn validate_host(host: &str) -> Result<(), ConfigError> {
    let host = host.trim();

    if host.is_empty() {
        return Err(ConfigError::InvalidHost(
            "the device address is not set".to_string(),
        ));
    }

    if host.contains("://") {
        return Err(ConfigError::InvalidHost(
            "the address must be host or host:port, without a scheme".to_string(),
        ));
    }

    if host.contains('/') || host.contains('\\') {
        return Err(ConfigError::InvalidHost(
            "the address must not contain slashes".to_string(),
        ));
    }

    if host.chars().any(char::is_whitespace) {
        return Err(ConfigError::InvalidHost(
            "the address must not contain spaces".to_string(),
        ));
    }

    if host.chars().any(|ch| matches!(ch, '@' | '?' | '#')) {
        return Err(ConfigError::InvalidHost(
            "the address must contain only host or host:port".to_string(),
        ));
    }

    Ok(())
}

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Serde(serde_json::Error),
    InvalidHost(String),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Config {
        Config {
            device: DeviceConfig {
                host: DEFAULT_HOST.to_string(),
            },
            scan_dir: PathBuf::from("/home/user/Documents/scans"),
        }
    }

    #[test]
    fn json_round_trip() {
        let config = sample();
        let json = serde_json::to_string_pretty(&config).expect("config serializes");
        let parsed = Config::from_json_str(&json).expect("config parses");

        assert_eq!(parsed, config);
    }

    #[test]
    fn unknown_fields_are_ignored() {
        let json = r#"{
            "device": { "host": "192.168.187.36", "futureField": 1 },
            "scanDir": "/tmp/scans",
            "anotherFutureField": { "nested": true }
        }"#;

        let config = Config::from_json_str(json).expect("unknown fields are ignored");

        assert_eq!(config.device.host, DEFAULT_HOST);
        assert_eq!(config.scan_dir, PathBuf::from("/tmp/scans"));
    }

    #[test]
    fn broken_json_is_an_error() {
        let error = Config::from_json_str("{ not json").expect_err("broken json must fail");

        assert!(matches!(error, ConfigError::Serde(_)));
    }

    #[test]
    fn empty_host_is_an_error() {
        let json = r#"{"device":{"host":""},"scanDir":"/tmp/scans"}"#;
        let error = Config::from_json_str(json).expect_err("empty host must fail");

        assert!(matches!(error, ConfigError::InvalidHost(_)));
    }

    #[test]
    fn host_with_scheme_or_slash_is_an_error() {
        for host in [
            "http://192.168.187.36",
            "192.168.187.36/eSCL",
            "192.168.1.1\\x",
            "user@192.168.1.1",
        ] {
            assert!(
                matches!(validate_host(host), Err(ConfigError::InvalidHost(_))),
                "host {host} must be rejected"
            );
        }

        let json = r#"{"device":{"host":"http://192.168.187.36"},"scanDir":"/tmp/scans"}"#;
        assert!(matches!(
            Config::from_json_str(json),
            Err(ConfigError::InvalidHost(_))
        ));
    }

    #[test]
    fn empty_scan_dir_is_an_error() {
        let json = r#"{"device":{"host":"192.168.187.36"},"scanDir":""}"#;
        let error = Config::from_json_str(json).expect_err("empty scan dir must fail");

        assert!(matches!(error, ConfigError::InvalidScanDir(_)));
    }

    #[test]
    fn defaults_use_documents_dir() {
        let config = Config::defaults_for(
            Path::new("/home/user"),
            Some(Path::new("/home/user/Documents")),
        );

        assert_eq!(config.device.host, DEFAULT_HOST);
        assert_eq!(config.scan_dir, PathBuf::from("/home/user/Documents/scans"));
    }

    #[test]
    fn defaults_fall_back_to_home_documents() {
        let config = Config::defaults_for(Path::new("/home/user"), None);

        assert_eq!(config.scan_dir, PathBuf::from("/home/user/Documents/scans"));
    }

    #[test]
    fn base_url_is_built_from_host() {
        let config = sample();
        let url = config.base_url().expect("base url");

        assert_eq!(url.as_str(), "http://192.168.187.36/eSCL/");
    }

    #[test]
    fn base_url_keeps_port() {
        let config = Config {
            device: DeviceConfig {
                host: "192.168.187.36:8080".to_string(),
            },
            scan_dir: PathBuf::from("/tmp/scans"),
        };
        let url = config.base_url().expect("base url");

        assert_eq!(url.as_str(), "http://192.168.187.36:8080/eSCL/");
        assert_eq!(url.port(), Some(8080));
    }

    #[test]
    fn load_or_create_reads_existing_file_and_creates_scan_dir() {
        let unique = format!(
            "prindeck-config-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0)
        );
        let root = std::env::temp_dir().join(unique);
        let path = root.join("prindeck").join("config.json");
        let scan_dir = root.join("scans");
        let json = format!(
            r#"{{"device":{{"host":"{}"}},"scanDir":"{}"}}"#,
            DEFAULT_HOST,
            scan_dir.display()
        );

        fs::create_dir_all(path.parent().expect("config parent")).expect("create config dir");
        fs::write(&path, json).expect("write config");

        let loaded = Config::load_or_create(&path).expect("config loads");

        assert_eq!(loaded.device.host, DEFAULT_HOST);
        assert_eq!(loaded.scan_dir, scan_dir);
        assert!(scan_dir.is_dir(), "scan dir must be created");

        let _ = fs::remove_dir_all(&root);
    }
}
