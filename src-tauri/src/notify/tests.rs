use super::measure_from_status;
use crate::status::DeviceStatus;

const HOST: &str = "192.168.187.36";

fn status(ok: bool, toner: Option<u32>, drum: Option<u32>) -> DeviceStatus {
    DeviceStatus {
        ok,
        host: HOST.to_string(),
        toner_percent: toner,
        drum_percent: drum,
        pages_total: None,
        duplex_pages: None,
        drum_pages_left: None,
        polled_at: None,
        error: None,
    }
}

#[test]
fn measure_maps_the_status_snapshot() {
    let online = measure_from_status(&status(true, Some(14), Some(8)));
    assert_eq!(
        (online.online, online.toner, online.drum),
        (true, Some(14), Some(8))
    );

    let offline = measure_from_status(&status(false, Some(14), Some(8)));
    assert_eq!(
        (offline.online, offline.toner, offline.drum),
        (false, Some(14), Some(8))
    );

    let clamped = measure_from_status(&status(true, Some(300), None));
    assert_eq!(
        (clamped.online, clamped.toner, clamped.drum),
        (true, None, None)
    );
}
