mod blob;
mod client;

#[cfg(test)]
mod tests;

pub use blob::{BlobError, parse_records, parse_width_records};
pub use client::{SnmpClient, SnmpError};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SnmpCounters {
    pub toner_percent: Option<u32>,
    pub drum_percent: Option<u32>,
    pub pages_total: Option<u32>,
    pub duplex_pages: Option<u32>,
    pub drum_pages_left: Option<u32>,
    pub jam_count: Option<u32>,
}

type NodeParser = fn(&[u8]) -> Result<Vec<(u8, u32)>, BlobError>;

pub fn counters_from(
    maintenance: &[u8],
    counter: &[u8],
    next_care: &[u8],
    jam_count: Option<&[u8]>,
) -> SnmpCounters {
    let maintenance = parse_node("maintenance", maintenance, parse_records);
    let counter = parse_node("counter", counter, parse_records);
    let next_care = parse_node("next care", next_care, parse_records);

    SnmpCounters {
        toner_percent: record_value(&maintenance, 0x81),
        drum_percent: record_value(&maintenance, 0x41).map(|hundredths| hundredths / 100),
        pages_total: record_value(&maintenance, 0x11),
        duplex_pages: record_value(&counter, 0x06),
        drum_pages_left: record_value(&next_care, 0x82),
        jam_count: jam_count
            .map(|blob| parse_node("jam count", blob, parse_width_records))
            .and_then(|records| record_value(&records, 0xa1)),
    }
}

fn parse_node(name: &str, blob: &[u8], parse: NodeParser) -> Vec<(u8, u32)> {
    match parse(blob) {
        Ok(records) => records,
        Err(error) => {
            tracing::warn!(%error, "failed to parse the {name} snmp blob");
            Vec::new()
        }
    }
}

fn record_value(records: &[(u8, u32)], id: u8) -> Option<u32> {
    records
        .iter()
        .find(|(record_id, _)| *record_id == id)
        .map(|&(_, value)| value)
}
