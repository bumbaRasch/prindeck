use super::sample;
use crate::config::ConfigHandle;

#[test]
fn get_returns_the_current_config() {
    let handle = ConfigHandle::new(sample());

    assert_eq!(handle.get(), sample());
}

#[test]
fn set_swaps_the_whole_config() {
    let handle = ConfigHandle::new(sample());

    let mut updated = sample();
    updated.device.host = "10.0.0.9".to_string();
    updated.notify.enabled = false;
    handle.set(updated.clone());

    assert_eq!(handle.get(), updated);
}
