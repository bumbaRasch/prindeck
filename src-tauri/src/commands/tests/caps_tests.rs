use std::path::Path;

use reqwest::Url;
use serde_json::json;

use crate::commands::FALLBACK_BASE_URL;
use crate::commands::caps::{StartupProblem, resolve_loaded};
use crate::config::Config;
use crate::escl::EsclClient;

#[test]
fn fallback_base_url_is_valid() {
    let url = Url::parse(FALLBACK_BASE_URL).expect("fallback URL literal is valid");

    assert!(EsclClient::new(url).is_ok());
}

#[test]
fn load_failure_and_fallback_serialize_as_startup_problems() {
    let (_, _, problems) = resolve_loaded(Err(StartupProblem::load(
        "failed to load the config".to_string(),
    )));

    assert_eq!(
        serde_json::to_value(problems).unwrap(),
        json!([
            {
                "kind": "loadFailed",
                "field": null,
                "detail": "failed to load the config"
            },
            {
                "kind": "fallback",
                "field": null,
                "detail": "default settings are in use"
            }
        ])
    );
}

#[test]
fn invalid_startup_config_carries_its_field() {
    let mut config = Config::defaults_for(Path::new("/tmp"), None);
    config.device.host = "bad host".to_string();

    let (_, _, problems) = resolve_loaded(Ok(config));

    assert_eq!(
        serde_json::to_value(problems).unwrap(),
        json!([
            {
                "kind": "invalid",
                "field": "host",
                "detail": "invalid device address: the address must not contain spaces"
            },
            {
                "kind": "fallback",
                "field": null,
                "detail": "default settings are in use"
            }
        ])
    );
}

#[test]
fn valid_startup_config_has_no_problems() {
    let config = Config::defaults_for(Path::new("/tmp"), None);

    let (_, _, problems) = resolve_loaded(Ok(config));

    assert!(problems.is_empty());
}
