use super::hex;
use crate::snmp::{SnmpCounters, counters_from};

#[test]
fn maps_counters_from_three_nodes() {
    let maintenance = hex("6301040000000141010400000e1011010400001e4931010400000007\
                          6f010400000d4881010400000028ff");
    let counter = hex("00010400001e4906010400000f6fff");
    let next_care = hex("82010400001097ff");

    let counters = counters_from(&maintenance, &counter, &next_care, None);

    assert_eq!(
        counters,
        SnmpCounters {
            toner_percent: Some(40),
            drum_percent: Some(36),
            pages_total: Some(7753),
            duplex_pages: Some(3951),
            drum_pages_left: Some(4247),
            jam_count: None,
        }
    );
}

#[test]
fn maps_jam_count_from_the_fourth_node() {
    let maintenance = hex("6301040000000141010400000e1011010400001e4931010400000007\
                          6f010400000d4881010400000028ff");
    let counter = hex("00010400001e4906010400000f6fff");
    let next_care = hex("82010400001097ff");
    let jam_count = hex("000200000102000221020002220200022302000225020006a10400000008ff");

    let counters = counters_from(&maintenance, &counter, &next_care, Some(&jam_count));

    assert_eq!(counters.jam_count, Some(8));
}

#[test]
fn missing_jam_node_leaves_jam_count_none() {
    let maintenance = hex("6301040000000141010400000e1011010400001e4931010400000007\
                          6f010400000d4881010400000028ff");
    let counter = hex("00010400001e4906010400000f6fff");
    let next_care = hex("82010400001097ff");

    let counters = counters_from(&maintenance, &counter, &next_care, None);

    assert_eq!(counters.jam_count, None);
    assert_eq!(counters.pages_total, Some(7753));
}

#[test]
fn unknown_record_ids_are_skipped() {
    let blob = hex("00010400001e49ff");

    let counters = counters_from(&hex("ff"), &blob, &hex("ff"), None);
    assert_eq!(counters, SnmpCounters::default());
}

#[test]
fn unknown_jam_record_ids_are_skipped() {
    let only_unknown_ids = hex("0102000221020002ff");

    let counters = counters_from(&hex("ff"), &hex("ff"), &hex("ff"), Some(&only_unknown_ids));

    assert_eq!(counters, SnmpCounters::default());
}

#[test]
fn corrupted_node_leaves_other_nodes_populated() {
    let counter = hex("00010400001e4906010400000f6fff");
    let next_care = hex("82010400001097ff");
    let jam_count = hex("000200000102000221020002220200022302000225020006a10400000008ff");

    let counters = counters_from(&hex("11010400"), &counter, &next_care, Some(&jam_count));

    assert_eq!(
        counters,
        SnmpCounters {
            toner_percent: None,
            drum_percent: None,
            pages_total: None,
            duplex_pages: Some(3951),
            drum_pages_left: Some(4247),
            jam_count: Some(8),
        }
    );
}
