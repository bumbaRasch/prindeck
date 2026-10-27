use snmp2::{Oid, Value};

use super::hex;
use crate::snmp::SnmpClient;
use crate::snmp::client::{SnmpError, optional_octet_string, with_port};

#[test]
fn missing_jam_varbind_is_none_instead_of_an_error() {
    let missing_node = [1u64, 3, 6];
    let no_such_instance = [(Oid::from(&missing_node).unwrap(), Value::NoSuchInstance)];

    assert_eq!(optional_octet_string(&no_such_instance, 0), None);
    assert_eq!(optional_octet_string(&no_such_instance, 1), None);

    let blob = hex("01020002ff");
    let present = [(Oid::from(&missing_node).unwrap(), Value::OctetString(&blob))];

    assert_eq!(optional_octet_string(&present, 0), Some(&blob[..]));
}

#[test]
fn unreachable_host_is_a_transport_error() {
    let client = SnmpClient::new("127.0.0.1:9", "public");
    assert!(matches!(client.fetch(), Err(SnmpError::Transport(_))));
}

#[test]
fn snmp_address_gets_the_default_port() {
    assert_eq!(with_port("192.168.187.36"), "192.168.187.36:161");
    assert_eq!(with_port("printer.local:1161"), "printer.local:1161");
    assert_eq!(with_port("[fe80::1]"), "[fe80::1]:161");
    assert_eq!(with_port("[fe80::1]:1161"), "[fe80::1]:1161");
}
