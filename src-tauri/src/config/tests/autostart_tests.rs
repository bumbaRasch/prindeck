use std::path::Path;

use super::{sample, sample_json};
use crate::config::Config;

#[test]
fn autostart_defaults_to_off() {
    let config = Config::defaults_for(Path::new("/home/user"), None);

    assert!(!config.autostart);
}

#[test]
fn missing_autostart_key_stays_off() {
    let mut json = sample_json();
    let object = json.as_object_mut().expect("config json is an object");
    object.remove("autostart");

    let config =
        Config::from_json_str(&json.to_string()).expect("missing autostart uses the default");

    assert!(!config.autostart);
}

#[test]
fn autostart_round_trips_through_json() {
    let mut config = sample();
    config.autostart = true;

    let json = serde_json::to_string(&config).expect("config serializes");

    assert!(json.contains("\"autostart\":true"));
    let parsed = Config::from_json_str(&json).expect("config parses");
    assert!(parsed.autostart);
    assert_eq!(parsed, config);
}
