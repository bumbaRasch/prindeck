use super::{counters_from, parse_records, BlobError, SnmpCounters};

fn hex(value: &str) -> Vec<u8> {
    (0..value.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&value[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn parses_maintenance_blob() {
    let blob = hex("6301040000000141010400000e1011010400001e4931010400000007\
                    6f010400000d4881010400000028ff");
    let records = parse_records(&blob).unwrap();
    assert_eq!(
        records,
        vec![(0x63, 1), (0x41, 3600), (0x11, 7753), (0x31, 7), (0x6f, 3400), (0x81, 40)]
    );
}

#[test]
fn maps_counters_from_three_nodes() {
    let maintenance = hex("6301040000000141010400000e1011010400001e4931010400000007\
                          6f010400000d4881010400000028ff");
    let counter = hex("00010400001e4906010400000f6fff");
    let next_care = hex("82010400001097ff");

    let counters = counters_from(&maintenance, &counter, &next_care);

    assert_eq!(
        counters,
        SnmpCounters {
            toner_percent: Some(40),
            drum_percent: Some(36),
            pages_total: Some(7753),
            duplex_pages: Some(3951),
            drum_pages_left: Some(4247),
        }
    );
}

#[test]
fn truncated_blob_is_an_error() {
    assert_eq!(
        parse_records(&hex("11010400001e")),
        Err(BlobError::Truncated)
    );
    assert_eq!(
        parse_records(&hex("11010400001e49")),
        Err(BlobError::MissingTerminator)
    );
}

#[test]
fn unknown_record_ids_are_skipped() {
    let blob = hex("00010400001e49ff");

    assert_eq!(parse_records(&blob), Ok(vec![(0x00, 7753)]));

    let counters = counters_from(&hex("ff"), &blob, &hex("ff"));
    assert_eq!(counters, SnmpCounters::default());
}
