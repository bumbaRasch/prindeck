use super::{Config, ConfigError};

pub(super) fn validate_host(host: &str) -> Result<(), ConfigError> {
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

pub(super) fn validate_queue(queue: &str) -> Result<(), ConfigError> {
    if queue.trim().is_empty() {
        return Err(ConfigError::InvalidCupsQueue(
            "the CUPS queue name is not set".to_string(),
        ));
    }

    Ok(())
}

impl Config {
    pub fn base_url(&self) -> Result<reqwest::Url, ConfigError> {
        validate_host(&self.device.host)?;

        reqwest::Url::parse(&format!("http://{}/eSCL/", self.device.host.trim()))
            .map_err(|error| ConfigError::InvalidHost(format!("failed to build the URL: {error}")))
    }
}
