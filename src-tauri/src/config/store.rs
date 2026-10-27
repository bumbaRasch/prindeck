use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::{Config, ConfigError};

impl Config {
    pub fn default_path() -> Result<PathBuf, ConfigError> {
        let dir = dirs::config_dir().ok_or(ConfigError::NoConfigDir)?;
        Ok(dir.join("prindeck").join("config.json"))
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
}

fn default_config() -> Result<Config, ConfigError> {
    let home = dirs::home_dir().ok_or(ConfigError::NoHomeDir)?;
    let documents = dirs::document_dir();

    Ok(Config::defaults_for(&home, documents.as_deref()))
}
