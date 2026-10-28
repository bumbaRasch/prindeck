use std::fs;

use serde_json::json;

use super::{sample, sample_json};
use crate::config::host::validate_host;
use crate::config::{Config, ConfigError, DEFAULT_HOST};

#[test]
fn config_wire_keys_pin_the_typescript_mirror() {
    let json = serde_json::to_value(sample()).expect("config serializes");

    let root = json.as_object().expect("config is an object");
    let keys: Vec<&str> = root.keys().map(|key| key.as_str()).collect();
    assert_eq!(
        keys,
        [
            "cupsQueue",
            "device",
            "notify",
            "pollSeconds",
            "scanDir",
            "snmp",
            "toner"
        ],
        "ui/src/types.ts mirrors these exact keys"
    );

    let object_keys = |key: &str| -> Vec<&str> {
        json[key]
            .as_object()
            .unwrap_or_else(|| panic!("{key} is an object"))
            .keys()
            .map(|field| field.as_str())
            .collect()
    };

    assert_eq!(object_keys("device"), ["host"]);
    assert_eq!(object_keys("snmp"), ["community"]);
    assert_eq!(
        object_keys("notify"),
        ["drumPercent", "enabled", "tonerPercent"]
    );
    assert_eq!(object_keys("toner"), ["model", "yieldPages"]);
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
    let mut json = sample_json();
    json["futureField"] = json!({ "nested": true });
    json["device"]["futureField"] = json!(1);

    let config = Config::from_json_str(&json.to_string()).expect("unknown fields are ignored");

    assert_eq!(config, sample());
}

#[test]
fn broken_json_is_an_error() {
    let error = Config::from_json_str("{ not json").expect_err("broken json must fail");

    assert!(matches!(error, ConfigError::Serde(_)));
}

#[test]
fn empty_host_is_an_error() {
    let mut json = sample_json();
    json["device"]["host"] = json!("");
    let error = Config::from_json_str(&json.to_string()).expect_err("empty host must fail");

    assert!(matches!(error, ConfigError::InvalidHost(_)));
}

#[test]
fn host_with_scheme_or_slash_is_an_error() {
    for host in [
        "http://192.0.2.10",
        "192.0.2.10/eSCL",
        "192.168.1.1\\x",
        "user@192.168.1.1",
    ] {
        assert!(
            matches!(validate_host(host), Err(ConfigError::InvalidHost(_))),
            "host {host} must be rejected"
        );
    }

    let mut json = sample_json();
    json["device"]["host"] = json!("http://192.0.2.10");
    assert!(matches!(
        Config::from_json_str(&json.to_string()),
        Err(ConfigError::InvalidHost(_))
    ));
}

#[test]
fn empty_scan_dir_is_an_error() {
    let mut json = sample_json();
    json["scanDir"] = json!("");
    let error = Config::from_json_str(&json.to_string()).expect_err("empty scan dir must fail");

    assert!(matches!(error, ConfigError::InvalidScanDir(_)));
}

#[test]
fn empty_cups_queue_is_an_error() {
    let mut json = sample_json();
    json["cupsQueue"] = json!("");
    let error = Config::from_json_str(&json.to_string()).expect_err("empty cups queue must fail");

    assert!(matches!(error, ConfigError::InvalidCupsQueue(_)));
}

#[test]
fn invalid_host_is_still_a_hard_error() {
    let mut json = sample_json();
    json["device"]["host"] = json!("bad host");

    assert!(matches!(
        Config::from_json_str(&json.to_string()),
        Err(ConfigError::InvalidHost(_))
    ));
}

#[test]
fn explicit_empty_community_stays_empty() {
    let mut json = sample_json();
    json["snmp"]["community"] = json!("");
    let config = Config::from_json_str(&json.to_string()).expect("explicit empty community parses");

    assert_eq!(config.snmp.community, "");
    assert_eq!(config.snmp_community(), "");
}

#[test]
fn base_url_is_built_from_host() {
    let config = sample();
    let url = config.base_url().expect("base url");

    assert_eq!(url.as_str(), "http://192.0.2.10/eSCL/");
}

#[test]
fn base_url_keeps_port() {
    let mut config = sample();
    config.device.host = "192.0.2.10:8080".to_string();
    let url = config.base_url().expect("base url");

    assert_eq!(url.as_str(), "http://192.0.2.10:8080/eSCL/");
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

    let mut config = sample();
    config.scan_dir = scan_dir.clone();
    let json = serde_json::to_string(&config).expect("config serializes");

    fs::create_dir_all(path.parent().expect("config parent")).expect("create config dir");
    fs::write(&path, json).expect("write config");

    let loaded = Config::load_or_create(&path).expect("config loads");

    assert_eq!(loaded.device.host, DEFAULT_HOST);
    assert_eq!(loaded.scan_dir, scan_dir);
    assert!(scan_dir.is_dir(), "scan dir must be created");

    let _ = fs::remove_dir_all(&root);
}
