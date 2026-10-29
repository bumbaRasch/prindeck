use std::sync::RwLock;

use super::Config;

#[derive(Debug)]
pub struct ConfigHandle {
    config: RwLock<Config>,
}

impl ConfigHandle {
    pub fn new(config: Config) -> ConfigHandle {
        ConfigHandle {
            config: RwLock::new(config),
        }
    }

    pub fn get(&self) -> Config {
        self.config
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn set(&self, config: Config) {
        *self
            .config
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = config;
    }
}
