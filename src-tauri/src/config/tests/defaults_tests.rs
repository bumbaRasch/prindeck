use std::path::{Path, PathBuf};
use std::time::Duration;

use super::sample;
use crate::config::{
    Config, DEFAULT_CUPS_QUEUE, DEFAULT_HOST, DEFAULT_POLL_SECONDS, DEFAULT_SNMP_COMMUNITY,
};

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
fn poll_seconds_default_is_300() {
    let config = Config::defaults_for(Path::new("/home/user"), None);

    assert_eq!(config.poll_seconds, DEFAULT_POLL_SECONDS);
    assert_eq!(config.poll_interval(), Duration::from_secs(300));
}

#[test]
fn poll_seconds_below_minimum_is_clamped() {
    let mut config = sample();
    config.poll_seconds = 5;

    assert_eq!(config.poll_interval(), Duration::from_secs(60));
}

#[test]
fn poll_seconds_above_maximum_is_clamped() {
    let mut config = sample();
    config.poll_seconds = 999_999;

    assert_eq!(config.poll_interval(), Duration::from_secs(3600));
}

#[test]
fn poll_seconds_bounds_are_inclusive() {
    let mut low = sample();
    low.poll_seconds = 60;

    let mut high = sample();
    high.poll_seconds = 3600;

    assert_eq!(low.poll_interval(), Duration::from_secs(60));
    assert_eq!(high.poll_interval(), Duration::from_secs(3600));
}

#[test]
fn missing_new_keys_use_defaults() {
    let json = r#"{"device":{"host":"192.168.187.36"},"scanDir":"/tmp/scans"}"#;
    let config = Config::from_json_str(json).expect("missing keys use defaults");

    assert_eq!(config.snmp.community, DEFAULT_SNMP_COMMUNITY);
    assert_eq!(config.poll_seconds, DEFAULT_POLL_SECONDS);
    assert_eq!(config.cups_queue, DEFAULT_CUPS_QUEUE);
}

#[test]
fn wrong_typed_poll_seconds_is_repaired_to_default() {
    let json = r#"{"device":{"host":"10.0.0.5"},"scanDir":"/tmp/scans","pollSeconds":"ten"}"#;
    let config = Config::from_json_str(json).expect("wrong-typed pollSeconds is repaired");

    assert_eq!(config.device.host, "10.0.0.5");
    assert_eq!(config.scan_dir, PathBuf::from("/tmp/scans"));
    assert_eq!(config.poll_seconds, DEFAULT_POLL_SECONDS);
    assert_eq!(config.poll_interval(), Duration::from_secs(300));
}

#[test]
fn wrong_typed_community_is_repaired_to_default() {
    let json = r#"{"device":{"host":"10.0.0.5"},"scanDir":"/tmp/scans","snmp":{"community":42}}"#;
    let config = Config::from_json_str(json).expect("wrong-typed community is repaired");

    assert_eq!(config.device.host, "10.0.0.5");
    assert_eq!(config.scan_dir, PathBuf::from("/tmp/scans"));
    assert_eq!(config.snmp.community, DEFAULT_SNMP_COMMUNITY);
    assert_eq!(config.snmp_community(), DEFAULT_SNMP_COMMUNITY);
}
