use std::path::PathBuf;

use crate::commands::AppState;
use crate::commands::caps::rebuild_client;
use crate::commands::settings::{field_error, live_field_error};
use crate::config::{
    Config, ConfigError, DEFAULT_CUPS_QUEUE, DEFAULT_HOST, DEFAULT_POLL_SECONDS,
    DEFAULT_SNMP_COMMUNITY, DeviceConfig, FieldError, FieldErrorKind, NotifyConfig, SnmpConfig,
    TonerConfig,
};
use crate::escl::{EsclClient, InputCaps, ScanCaps};

fn config_with_host(host: &str) -> Config {
    Config {
        device: DeviceConfig {
            host: host.to_string(),
        },
        scan_dir: PathBuf::from("/tmp/scans"),
        cups_queue: DEFAULT_CUPS_QUEUE.to_string(),
        snmp: SnmpConfig {
            community: DEFAULT_SNMP_COMMUNITY.to_string(),
        },
        poll_seconds: DEFAULT_POLL_SECONDS,
        autostart: false,
        notify: NotifyConfig::defaults(),
        toner: TonerConfig::defaults(),
    }
}

fn state_for(config: &Config) -> AppState {
    let url = config.base_url().expect("base url");
    let client = EsclClient::new(url).expect("client builds");

    AppState::from_parts(config.clone(), client, None)
}

fn sample_caps() -> ScanCaps {
    ScanCaps {
        make_and_model: "Brother MFC-L2710DW series".to_string(),
        platen: InputCaps {
            max_width: 2550,
            max_height: 3507,
            resolutions: Vec::new(),
            color_modes: Vec::new(),
            formats: Vec::new(),
        },
        adf: None,
        feeder_capacity: None,
    }
}

#[test]
fn host_change_rebuilds_the_client_and_clears_the_caps_cache() {
    let state = state_for(&config_with_host(DEFAULT_HOST));
    *state
        .caps
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(sample_caps());

    let updated = config_with_host("10.0.0.9");
    let rebuilt = rebuild_client(&state, &updated).expect("client rebuilds for the new host");

    assert_eq!(rebuilt.base().as_str(), "http://10.0.0.9/eSCL/");
    assert_eq!(state.client().base().as_str(), "http://10.0.0.9/eSCL/");

    let caps = state
        .caps
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    assert_eq!(caps, None, "the caps cache must be reset");

    assert_eq!(
        state.config.get().device.host,
        DEFAULT_HOST,
        "the handle swap is set_config's job, not the rebuild's"
    );
}

#[test]
fn save_io_errors_are_whole_form_failures() {
    let error = field_error(ConfigError::Io(std::io::Error::other("disk full")));

    assert_eq!(
        error,
        FieldError {
            field: None,
            kind: FieldErrorKind::Io,
        }
    );
}

#[test]
fn client_rebuild_errors_are_whole_form_failures() {
    let error = live_field_error("client rebuild failed".to_string());

    assert_eq!(
        error,
        FieldError {
            field: None,
            kind: FieldErrorKind::Io,
        }
    );
}
