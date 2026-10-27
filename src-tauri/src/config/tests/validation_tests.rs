use std::fs;
use std::path::PathBuf;

use super::sample;
use crate::config::host::validate_host;
use crate::config::{
    Config, ConfigError, DEFAULT_CUPS_QUEUE, DEFAULT_HOST, DEFAULT_POLL_SECONDS,
    DEFAULT_SNMP_COMMUNITY, DeviceConfig, SnmpConfig,
};

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
fn empty_cups_queue_is_an_error() {
    let json = r#"{"device":{"host":"192.168.187.36"},"scanDir":"/tmp/scans","cupsQueue":""}"#;
    let error = Config::from_json_str(json).expect_err("empty cups queue must fail");

    assert!(matches!(error, ConfigError::InvalidCupsQueue(_)));
}

#[test]
fn invalid_host_is_still_a_hard_error() {
    let json = r#"{"device":{"host":"bad host"},"scanDir":"/tmp/scans"}"#;

    assert!(matches!(
        Config::from_json_str(json),
        Err(ConfigError::InvalidHost(_))
    ));
}

#[test]
fn explicit_empty_community_stays_empty() {
    let json =
        r#"{"device":{"host":"192.168.187.36"},"scanDir":"/tmp/scans","snmp":{"community":""}}"#;
    let config = Config::from_json_str(json).expect("explicit empty community parses");

    assert_eq!(config.snmp.community, "");
    assert_eq!(config.snmp_community(), "");
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
        cups_queue: DEFAULT_CUPS_QUEUE.to_string(),
        snmp: SnmpConfig {
            community: DEFAULT_SNMP_COMMUNITY.to_string(),
        },
        poll_seconds: DEFAULT_POLL_SECONDS,
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
