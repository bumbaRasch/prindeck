use serde_json::json;

use crate::commands::{ScanDone, ScanError, user_message};
use crate::escl::EsclError;

fn sample_error(status: u16, body: &str) -> EsclError {
    EsclError::Http {
        status,
        body: body.to_string(),
    }
}

#[test]
fn user_message_maps_device_errors() {
    assert_eq!(user_message(&sample_error(503, "")), "the device is busy");
    assert_eq!(
        user_message(&sample_error(409, "")),
        "the device does not support this combination of settings"
    );
    assert_eq!(
        user_message(&sample_error(500, "<e>Feeder empty</e>")),
        "the feeder is out of paper"
    );
    assert_eq!(
        user_message(&EsclError::Timeout),
        "the device is not responding"
    );
    assert_eq!(
        user_message(&EsclError::Transport("connect refused".to_string())),
        "no connection to the device"
    );
    assert_eq!(
        user_message(&EsclError::InvalidParams(
            "the device does not support 150 dpi".to_string()
        )),
        "the device does not support 150 dpi"
    );
}

#[test]
fn user_message_keeps_http_code_for_other_statuses() {
    assert_eq!(
        user_message(&sample_error(418, "")),
        "the device returned HTTP 418"
    );
    assert_eq!(
        user_message(&sample_error(500, "<e>internal</e>")),
        "the device returned HTTP 500"
    );
}

#[test]
fn serializes_done_and_error_payloads() {
    let job_id = "/eSCL/ScanJobs/abc";

    let done = serde_json::to_value(ScanDone {
        job_id: job_id.to_string(),
        file: "/tmp/scans/scan-20261027-141530.pdf".to_string(),
    })
    .expect("done serializes");
    assert_eq!(
        done,
        json!({
            "jobId": job_id,
            "file": "/tmp/scans/scan-20261027-141530.pdf"
        })
    );

    let error = serde_json::to_value(ScanError {
        job_id: job_id.to_string(),
        message: "the device is busy".to_string(),
    })
    .expect("error serializes");
    assert_eq!(
        error,
        json!({ "jobId": job_id, "message": "the device is busy" })
    );
}
