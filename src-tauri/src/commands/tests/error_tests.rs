use serde_json::json;

use crate::commands::{ScanDone, ScanError, ScanErrorKind, ScanFailure, user_message};
use crate::escl::EsclError;

fn sample_error(status: u16, body: &str) -> EsclError {
    EsclError::Http {
        status,
        body: body.to_string(),
    }
}

fn kind_of(error: &EsclError) -> ScanErrorKind {
    ScanFailure::from_escl(error).kind
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
fn classifies_scan_error_kinds() {
    assert_eq!(kind_of(&EsclError::Timeout), ScanErrorKind::Transport);
    assert_eq!(
        kind_of(&EsclError::Transport("connect refused".to_string())),
        ScanErrorKind::Transport
    );
    assert_eq!(kind_of(&sample_error(503, "")), ScanErrorKind::Busy);
    assert_eq!(kind_of(&sample_error(409, "")), ScanErrorKind::Invalid);
    assert_eq!(
        kind_of(&EsclError::InvalidParams(
            "the device does not support 150 dpi".to_string()
        )),
        ScanErrorKind::Invalid
    );
    assert_eq!(
        kind_of(&EsclError::InvalidUrl("bad url".to_string())),
        ScanErrorKind::Invalid
    );
    assert_eq!(
        kind_of(&sample_error(500, "<e>Feeder empty</e>")),
        ScanErrorKind::Device
    );
    assert_eq!(kind_of(&sample_error(418, "")), ScanErrorKind::Device);
    assert_eq!(
        kind_of(&EsclError::Parse("bad xml".to_string())),
        ScanErrorKind::Device
    );
}

#[test]
fn scan_failure_serializes_kind_and_message() {
    let failure = ScanFailure::from_escl(&EsclError::Timeout);
    let value = serde_json::to_value(&failure).expect("failure serializes");

    assert_eq!(
        value,
        json!({ "kind": "transport", "message": "the device is not responding" })
    );
}

#[test]
fn serializes_done_and_error_payloads() {
    let job_id = "/eSCL/ScanJobs/abc";

    let done = serde_json::to_value(ScanDone {
        job_id: job_id.to_string(),
        files: vec![
            "/tmp/scans/scan-20261027-141530-p1.jpg".to_string(),
            "/tmp/scans/scan-20261027-141530-p2.jpg".to_string(),
        ],
        partial: true,
        warning: Some("the device is not responding".to_string()),
    })
    .expect("done serializes");
    assert_eq!(
        done,
        json!({
            "jobId": job_id,
            "files": [
                "/tmp/scans/scan-20261027-141530-p1.jpg",
                "/tmp/scans/scan-20261027-141530-p2.jpg"
            ],
            "partial": true,
            "warning": "the device is not responding"
        })
    );

    let clean = serde_json::to_value(ScanDone {
        job_id: job_id.to_string(),
        files: vec!["/tmp/scans/scan-20261027-141530.pdf".to_string()],
        partial: false,
        warning: None,
    })
    .expect("clean done serializes");
    assert_eq!(
        clean,
        json!({
            "jobId": job_id,
            "files": ["/tmp/scans/scan-20261027-141530.pdf"],
            "partial": false,
            "warning": null
        })
    );

    let error = serde_json::to_value(ScanError {
        job_id: job_id.to_string(),
        kind: ScanErrorKind::Busy,
        message: "the device is busy".to_string(),
    })
    .expect("error serializes");
    assert_eq!(
        error,
        json!({
            "jobId": job_id,
            "kind": "busy",
            "message": "the device is busy"
        })
    );
}
