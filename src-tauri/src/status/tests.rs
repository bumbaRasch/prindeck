use std::path::Path;
use std::time::Duration;

use chrono::DateTime;

use super::poll::next_delay;
use super::{StatusCache, now_rfc3339};
use crate::config::{Config, ConfigHandle};
use crate::snmp::SnmpCounters;

const HOST: &str = "192.168.187.36";

fn counters() -> SnmpCounters {
    SnmpCounters {
        toner_percent: Some(40),
        drum_percent: Some(80),
        pages_total: Some(7753),
        duplex_pages: Some(3967),
        drum_pages_left: Some(4247),
    }
}

#[test]
fn failed_poll_keeps_the_last_good_values() {
    let cache = StatusCache::new(HOST.to_string());
    cache.apply_success(counters());
    let good = cache.snapshot();

    cache.apply_failure("snmp transport error: timeout".to_string());
    let failed = cache.snapshot();

    assert!(!failed.ok);
    assert_eq!(
        failed.error.as_deref(),
        Some("snmp transport error: timeout")
    );
    assert_eq!(failed.polled_at, good.polled_at);
    assert_eq!(failed.toner_percent, good.toner_percent);
    assert_eq!(failed.drum_percent, good.drum_percent);
    assert_eq!(failed.pages_total, good.pages_total);
    assert_eq!(failed.duplex_pages, good.duplex_pages);
    assert_eq!(failed.drum_pages_left, good.drum_pages_left);
}

#[test]
fn first_snapshot_is_empty_and_not_ok() {
    let cache = StatusCache::new(HOST.to_string());
    let snapshot = cache.snapshot();

    assert_eq!(snapshot.host, HOST);
    assert!(!snapshot.ok);
    assert_eq!(snapshot.polled_at, None);
    assert_eq!(snapshot.error, None);
    assert_eq!(snapshot.toner_percent, None);
    assert_eq!(snapshot.drum_percent, None);
    assert_eq!(snapshot.pages_total, None);
    assert_eq!(snapshot.duplex_pages, None);
    assert_eq!(snapshot.drum_pages_left, None);
}

#[test]
fn timestamp_is_rfc3339_utc() {
    let parsed = DateTime::parse_from_rfc3339(&now_rfc3339()).unwrap();

    assert_eq!(parsed.offset().local_minus_utc(), 0);
}

fn config_with_poll_seconds(seconds: u64) -> Config {
    let mut config = Config::defaults_for(Path::new("/home/user"), None);
    config.poll_seconds = seconds;
    config
}

#[test]
fn next_delay_rereads_the_interval_per_tick() {
    let handle = ConfigHandle::new(config_with_poll_seconds(5));

    assert_eq!(
        next_delay(&handle),
        Duration::from_secs(60),
        "a below-minimum interval keeps the documented clamp"
    );

    handle.set(config_with_poll_seconds(300));

    assert_eq!(next_delay(&handle), Duration::from_secs(300));
}
