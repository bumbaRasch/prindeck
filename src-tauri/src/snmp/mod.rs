mod blob;

#[cfg(test)]
mod tests;

pub use blob::{BlobError, parse_records};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SnmpCounters {
    pub toner_percent: Option<u32>,
    pub drum_percent: Option<u32>,
    pub pages_total: Option<u32>,
    pub duplex_pages: Option<u32>,
    pub drum_pages_left: Option<u32>,
}

pub fn counters_from(maintenance: &[u8], counter: &[u8], next_care: &[u8]) -> SnmpCounters {
    let maintenance = parse_node("maintenance", maintenance);
    let counter = parse_node("counter", counter);
    let next_care = parse_node("next care", next_care);

    SnmpCounters {
        toner_percent: record_value(&maintenance, 0x81),
        drum_percent: record_value(&maintenance, 0x41).map(|hundredths| hundredths / 100),
        pages_total: record_value(&maintenance, 0x11),
        duplex_pages: record_value(&counter, 0x06),
        drum_pages_left: record_value(&next_care, 0x82),
    }
}

fn parse_node(name: &str, blob: &[u8]) -> Vec<(u8, u32)> {
    match parse_records(blob) {
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
