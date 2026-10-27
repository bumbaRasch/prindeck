use std::error::Error;
use std::fmt;
use std::time::Duration;

use snmp2::{Oid, SyncSession, Value};

use super::{SnmpCounters, counters_from};

const MAINTENANCE_OID: [u64; 17] = [1, 3, 6, 1, 4, 1, 2435, 2, 3, 9, 4, 2, 1, 5, 5, 8, 0];
const COUNTER_OID: [u64; 17] = [1, 3, 6, 1, 4, 1, 2435, 2, 3, 9, 4, 2, 1, 5, 5, 10, 0];
const NEXT_CARE_OID: [u64; 17] = [1, 3, 6, 1, 4, 1, 2435, 2, 3, 9, 4, 2, 1, 5, 5, 11, 0];
const SNMP_PORT: u16 = 161;
const TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub struct SnmpClient {
    address: String,
    community: String,
    timeout: Duration,
}

#[derive(Debug)]
pub enum SnmpError {
    Transport(String),
    Protocol(String),
}

impl fmt::Display for SnmpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SnmpError::Transport(message) => write!(f, "snmp transport error: {message}"),
            SnmpError::Protocol(message) => write!(f, "snmp protocol error: {message}"),
        }
    }
}

impl Error for SnmpError {}

impl SnmpClient {
    pub fn new(host: &str, community: &str) -> SnmpClient {
        SnmpClient {
            address: with_port(host),
            community: community.to_owned(),
            timeout: TIMEOUT,
        }
    }

    pub fn fetch(&self) -> Result<SnmpCounters, SnmpError> {
        let mut session = SyncSession::new_v2c(
            &self.address,
            self.community.as_bytes(),
            Some(self.timeout),
            0,
        )
        .map_err(|error| SnmpError::Transport(error.to_string()))?;
        let maintenance_oid = oid(&MAINTENANCE_OID)?;
        let counter_oid = oid(&COUNTER_OID)?;
        let next_care_oid = oid(&NEXT_CARE_OID)?;
        let response = session
            .get_many(&[&maintenance_oid, &counter_oid, &next_care_oid])
            .map_err(|error| SnmpError::Transport(error.to_string()))?;
        let varbinds: Vec<_> = response.varbinds.collect();
        let maintenance = octet_string(&varbinds, 0)?;
        let counter = octet_string(&varbinds, 1)?;
        let next_care = octet_string(&varbinds, 2)?;
        Ok(counters_from(maintenance, counter, next_care))
    }
}

pub(super) fn with_port(host: &str) -> String {
    let already_ported = if host.contains('[') {
        host.contains("]:")
    } else {
        host.contains(':')
    };

    if already_ported {
        host.to_owned()
    } else {
        format!("{host}:{SNMP_PORT}")
    }
}

fn oid(components: &[u64]) -> Result<Oid<'static>, SnmpError> {
    Oid::from(components)
        .map_err(|_| SnmpError::Protocol(format!("malformed snmp oid {components:?}")))
}

fn octet_string<'a, 'b>(
    varbinds: &'a [(Oid<'b>, Value<'b>)],
    index: usize,
) -> Result<&'a [u8], SnmpError> {
    match varbinds.get(index) {
        Some((_, Value::OctetString(blob))) => Ok(blob),
        _ => Err(SnmpError::Protocol(format!(
            "the snmp response varbind {index} is not an octet string"
        ))),
    }
}
