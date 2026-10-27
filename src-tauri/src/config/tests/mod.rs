mod defaults_tests;
mod handle_tests;
mod notify_tests;
mod validation_tests;

use std::path::PathBuf;

use super::{
    Config, DEFAULT_CUPS_QUEUE, DEFAULT_HOST, DEFAULT_POLL_SECONDS, DEFAULT_SNMP_COMMUNITY,
    DeviceConfig, NotifyConfig, SnmpConfig, TonerConfig,
};

fn sample() -> Config {
    Config {
        device: DeviceConfig {
            host: DEFAULT_HOST.to_string(),
        },
        scan_dir: PathBuf::from("/home/user/Documents/scans"),
        cups_queue: DEFAULT_CUPS_QUEUE.to_string(),
        snmp: SnmpConfig {
            community: DEFAULT_SNMP_COMMUNITY.to_string(),
        },
        poll_seconds: DEFAULT_POLL_SECONDS,
        notify: NotifyConfig::defaults(),
        toner: TonerConfig::defaults(),
    }
}

fn sample_json() -> serde_json::Value {
    serde_json::to_value(sample()).expect("config serializes")
}
