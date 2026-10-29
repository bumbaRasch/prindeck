use serde_json::json;

use super::sample_time;
use crate::commands::scan::{StartRejection, rejection_failure, validate_job_id};
use crate::commands::scan_files::{format_matches, scan_file_name};
use crate::commands::{ScanErrorKind, ScanPhase, ScanProgress};
use crate::escl::DocFormat;

#[test]
fn busy_refusal_maps_to_busy_kind() {
    let failure = rejection_failure(StartRejection::Busy);

    assert_eq!(failure.kind, ScanErrorKind::Busy);
    assert_eq!(failure.message, "the device is busy");
}

#[test]
fn duplicate_job_maps_to_invalid_kind() {
    let failure = rejection_failure(StartRejection::DuplicateJob);

    assert_eq!(failure.kind, ScanErrorKind::Invalid);
    assert_eq!(failure.message, "this job is already running");
}

#[test]
fn validates_job_ids() {
    for valid in [
        "/eSCL/ScanJobs/8f7c1c2e-0b1a-4b7e-9b0a-1c2d3e4f5a6b",
        "/eSCL/ScanJobs/uuid_like-123",
    ] {
        assert!(validate_job_id(valid).is_ok(), "{valid} must be accepted");
    }

    for invalid in [
        "",
        "/eSCL/ScanJobs/",
        "http://evil.example/eSCL/ScanJobs/abc",
        "/eSCL/ScanJobs/../etc/passwd",
        "/eSCL/ScanJobs/..",
        "/eSCL/ScanJobs/abc/def",
        "/eSCL/ScanJobs/abc?x=1",
        "/eSCL/ScanJobs/abc#frag",
        "/eSCL/ScanJobs/abc def",
        "/eSCL/ScanJobs/abc\\def",
        "/eSCL/ScanJobs/http://evil",
        "eSCL/ScanJobs/abc",
    ] {
        assert!(
            validate_job_id(invalid).is_err(),
            "{invalid} must be rejected"
        );
    }
}

#[test]
fn builds_scan_file_names() {
    assert_eq!(
        scan_file_name(sample_time(), DocFormat::Pdf),
        "scan-20261027-141530.pdf"
    );
    assert_eq!(
        scan_file_name(sample_time(), DocFormat::Jpeg),
        "scan-20261027-141530.jpg"
    );
}

#[test]
fn detects_file_magic() {
    assert!(format_matches(DocFormat::Pdf, b"%PDF-1.7\n..."));
    assert!(format_matches(
        DocFormat::Jpeg,
        &[0xFF, 0xD8, 0xFF, 0xE0, 0x00]
    ));
    assert!(!format_matches(DocFormat::Pdf, &[0xFF, 0xD8, 0xFF]));
    assert!(!format_matches(DocFormat::Jpeg, b"%PDF-1.7"));
    assert!(!format_matches(DocFormat::Pdf, b"\x89PNG\r\n\x1a\n"));
    assert!(!format_matches(DocFormat::Pdf, b""));
    assert!(!format_matches(DocFormat::Jpeg, b"garbage"));
}

#[test]
fn serializes_progress_payloads_camel_case() {
    let job_id = "/eSCL/ScanJobs/8f7c1c2e-0b1a-4b7e-9b0a-1c2d3e4f5a6b";

    for (phase, expected) in [
        (ScanPhase::Created, "created"),
        (ScanPhase::Scanning, "scanning"),
        (ScanPhase::Done, "done"),
        (ScanPhase::Canceled, "canceled"),
    ] {
        let payload = ScanProgress {
            job_id: job_id.to_string(),
            phase,
            pages: 0,
        };
        let value = serde_json::to_value(&payload).expect("progress serializes");

        assert_eq!(
            value,
            json!({ "jobId": job_id, "phase": expected, "pages": 0 })
        );
    }
}
