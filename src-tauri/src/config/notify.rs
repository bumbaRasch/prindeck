use serde::{Deserialize, Serialize};

use super::ConfigError;

pub const DEFAULT_TONER_THRESHOLD_PERCENT: u8 = 15;
pub const DEFAULT_DRUM_THRESHOLD_PERCENT: u8 = 10;
pub const MAX_PERCENT: u8 = 100;
pub const MAX_YIELD_PAGES: u32 = 100_000;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotifyConfig {
    pub enabled: bool,
    pub toner_percent: u8,
    pub drum_percent: u8,
}

impl NotifyConfig {
    pub fn defaults() -> NotifyConfig {
        NotifyConfig {
            enabled: true,
            toner_percent: DEFAULT_TONER_THRESHOLD_PERCENT,
            drum_percent: DEFAULT_DRUM_THRESHOLD_PERCENT,
        }
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.toner_percent > MAX_PERCENT {
            return Err(ConfigError::InvalidNotify(format!(
                "notify.tonerPercent must be at most {MAX_PERCENT}, got {}",
                self.toner_percent
            )));
        }

        if self.drum_percent > MAX_PERCENT {
            return Err(ConfigError::InvalidNotify(format!(
                "notify.drumPercent must be at most {MAX_PERCENT}, got {}",
                self.drum_percent
            )));
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TonerConfig {
    pub model: Option<String>,
    pub yield_pages: Option<u32>,
}

impl TonerConfig {
    pub fn defaults() -> TonerConfig {
        TonerConfig {
            model: None,
            yield_pages: None,
        }
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if let Some(pages) = self.yield_pages
            && pages > MAX_YIELD_PAGES
        {
            return Err(ConfigError::InvalidToner(format!(
                "toner.yieldPages must be at most {MAX_YIELD_PAGES}, got {pages}"
            )));
        }

        Ok(())
    }
}
