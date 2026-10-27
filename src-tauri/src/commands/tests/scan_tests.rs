use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local, TimeZone};
use serde_json::json;

use crate::commands::scan::validate_job_id;
use crate::commands::scan_files::{format_matches, part_path, scan_file_name, unique_scan_path};
use crate::commands::{ScanPhase, ScanProgress};
use crate::escl::DocFormat;

fn sample_time() -> DateTime<Local> {
    Local
        .with_ymd_and_hms(2026, 10, 27, 14, 15, 30)
        .single()
        .expect("valid local time")
}

fn unique_temp_dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!(
        "prindeck-commands-{name}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
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
fn avoids_existing_scan_file_names() {
    let dir = unique_temp_dir("names");

    let first = unique_scan_path(&dir, sample_time(), DocFormat::Pdf);
    assert_eq!(
        first.file_name().and_then(|name| name.to_str()),
        Some("scan-20261027-141530.pdf")
    );

    fs::write(&first, b"%PDF-1.4").expect("write first file");
    let second = unique_scan_path(&dir, sample_time(), DocFormat::Pdf);
    assert_eq!(
        second.file_name().and_then(|name| name.to_str()),
        Some("scan-20261027-141530-2.pdf")
    );

    fs::write(&second, b"%PDF-1.4").expect("write second file");
    let third = unique_scan_path(&dir, sample_time(), DocFormat::Pdf);
    assert_eq!(
        third.file_name().and_then(|name| name.to_str()),
        Some("scan-20261027-141530-3.pdf")
    );

    let _ = fs::remove_dir_all(&dir);
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

#[test]
fn builds_part_path_from_job_id() {
    let part = part_path(Path::new("/tmp/scans"), "/eSCL/ScanJobs/abc-123");

    assert_eq!(
        part.file_name().and_then(|name| name.to_str()),
        Some(".scan-abc-123.part")
    );

    let weird = part_path(Path::new("/tmp/scans"), "/eSCL/ScanJobs/../../eth");
    assert_eq!(
        weird.file_name().and_then(|name| name.to_str()),
        Some(".scan-eth.part")
    );
}
