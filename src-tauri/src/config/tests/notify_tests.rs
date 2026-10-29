use serde_json::json;

use super::sample_json;
use crate::config::{Config, ConfigError, NotifyConfig};

#[test]
fn config_without_notify_names_the_missing_key() {
    let json = r#"{"device":{"host":"192.0.2.10"},"scanDir":"/tmp/scans","toner":{"model":null,"yieldPages":null}}"#;
    let error = Config::from_json_str(json).expect_err("a config without notify must fail");

    assert!(
        error.to_string().contains("missing field `notify`"),
        "the error must name the absent key, got: {error}"
    );
}

#[test]
fn config_without_toner_names_the_missing_key() {
    let json = r#"{"device":{"host":"192.0.2.10"},"scanDir":"/tmp/scans","notify":{"enabled":true,"tonerPercent":15,"drumPercent":10}}"#;
    let error = Config::from_json_str(json).expect_err("a config without toner must fail");

    assert!(
        error.to_string().contains("missing field `toner`"),
        "the error must name the absent key, got: {error}"
    );
}

#[test]
fn toner_percent_above_100_is_rejected() {
    let mut json = sample_json();
    json["notify"]["tonerPercent"] = json!(200);
    let error =
        Config::from_json_str(&json.to_string()).expect_err("tonerPercent above 100 must fail");

    assert!(matches!(error, ConfigError::InvalidNotify(_)));
    assert!(
        error.to_string().contains("notify.tonerPercent"),
        "the error must name the offending key, got: {error}"
    );
}

#[test]
fn drum_percent_above_100_is_rejected() {
    let mut json = sample_json();
    json["notify"]["drumPercent"] = json!(200);
    let error =
        Config::from_json_str(&json.to_string()).expect_err("drumPercent above 100 must fail");

    assert!(matches!(error, ConfigError::InvalidNotify(_)));
    assert!(
        error.to_string().contains("notify.drumPercent"),
        "the error must name the offending key, got: {error}"
    );
}

#[test]
fn yield_pages_above_100000_is_rejected() {
    let mut json = sample_json();
    json["toner"]["yieldPages"] = json!(100_001);
    let error =
        Config::from_json_str(&json.to_string()).expect_err("yieldPages above 100000 must fail");

    assert!(matches!(error, ConfigError::InvalidToner(_)));
    assert!(
        error.to_string().contains("toner.yieldPages"),
        "the error must name the offending key, got: {error}"
    );
}

#[test]
fn notify_and_toner_bounds_are_inclusive() {
    let mut json = sample_json();
    json["notify"]["tonerPercent"] = json!(100);
    json["notify"]["drumPercent"] = json!(100);
    json["toner"]["yieldPages"] = json!(100_000);

    assert!(Config::from_json_str(&json.to_string()).is_ok());
}

#[test]
fn notify_and_toner_parse_from_json() {
    let mut json = sample_json();
    json["notify"] = json!({ "enabled": false, "tonerPercent": 20, "drumPercent": 5 });
    json["toner"] = json!({ "model": "TN-2420", "yieldPages": 3000 });

    let config = Config::from_json_str(&json.to_string()).expect("notify and toner parse");

    assert_eq!(
        config.notify,
        NotifyConfig {
            enabled: false,
            toner_percent: 20,
            drum_percent: 5,
        }
    );
    assert_eq!(config.toner.model.as_deref(), Some("TN-2420"));
    assert_eq!(config.toner.yield_pages, Some(3000));
}
