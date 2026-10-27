mod defaults_tests;
mod validation_tests;

use std::path::PathBuf;

use super::{
    Config, DEFAULT_HOST, DEFAULT_POLL_SECONDS, DEFAULT_SNMP_COMMUNITY, DeviceConfig, SnmpConfig,
};

fn sample() -> Config {
    Config {
        device: DeviceConfig {
            host: DEFAULT_HOST.to_string(),
        },
        scan_dir: PathBuf::from("/home/user/Documents/scans"),
        snmp: SnmpConfig {
            community: DEFAULT_SNMP_COMMUNITY.to_string(),
        },
        poll_seconds: DEFAULT_POLL_SECONDS,
    }
}
