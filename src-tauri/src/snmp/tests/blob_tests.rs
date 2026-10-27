use super::hex;
use crate::snmp::{BlobError, parse_records, parse_width_records};

#[test]
fn parses_maintenance_blob() {
    let blob = hex("6301040000000141010400000e1011010400001e4931010400000007\
                    6f010400000d4881010400000028ff");
    let records = parse_records(&blob).unwrap();
    assert_eq!(
        records,
        vec![
            (0x63, 1),
            (0x41, 3600),
            (0x11, 7753),
            (0x31, 7),
            (0x6f, 3400),
            (0x81, 40)
        ]
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
fn unsupported_record_header_is_an_error() {
    let blob = hex("a10400000008ff");

    assert_eq!(parse_records(&blob), Err(BlobError::BadHeader { id: 0xa1 }));
}

#[test]
fn trailing_bad_header_beats_truncation() {
    assert_eq!(
        parse_records(&hex("a10208")),
        Err(BlobError::BadHeader { id: 0xa1 })
    );
    assert_eq!(parse_records(&hex("a102")), Err(BlobError::Truncated));
}

#[test]
fn parses_jam_count_blob() {
    let blob = hex("000200000102000221020002220200022302000225020006a10400000008ff");

    let records = parse_width_records(&blob).unwrap();

    assert_eq!(
        records,
        vec![
            (0x00, 0),
            (0x01, 2),
            (0x21, 2),
            (0x22, 2),
            (0x23, 2),
            (0x25, 6),
            (0xa1, 8)
        ]
    );
}

#[test]
fn malformed_jam_blob_is_an_error() {
    assert_eq!(
        parse_width_records(&hex("a100ff")),
        Err(BlobError::BadHeader { id: 0xa1 })
    );
    assert_eq!(
        parse_width_records(&hex("a1050000000000ff")),
        Err(BlobError::BadHeader { id: 0xa1 })
    );
    assert_eq!(
        parse_width_records(&hex("a1040000")),
        Err(BlobError::Truncated)
    );
    assert_eq!(
        parse_width_records(&hex("01020002")),
        Err(BlobError::MissingTerminator)
    );
}
