use std::io;
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

use super::{
    PRINT_CANCEL_FAILED_LABEL, PRINT_COPIES_RANGE_LABEL, PRINT_CUPS_MISSING_LABEL,
    PRINT_QUEUE_FAILED_LABEL, PRINT_QUEUE_MISSING_LABEL, PRINT_SEND_FAILED_LABEL,
    PRINT_SIDES_UNKNOWN_LABEL, check_copies, job_belongs_to_queue, print_message, sides_from_ipc,
};
use crate::print::{PrintError, Sides};

const QUEUE: &str = "Brother_MFC_L2710DW_series";

fn failed(program: &str, stderr: &str) -> PrintError {
    PrintError::Failed {
        program: program.to_string(),
        status: ExitStatus::from_raw(256),
        stderr: stderr.to_string(),
    }
}

#[test]
fn sides_mapping_accepts_cups_strings() {
    assert_eq!(sides_from_ipc("one-sided"), Ok(Sides::OneSided));
    assert_eq!(sides_from_ipc("two-sided-long-edge"), Ok(Sides::LongEdge));
    assert_eq!(sides_from_ipc("two-sided-short-edge"), Ok(Sides::ShortEdge));
}

#[test]
fn sides_mapping_rejects_unknown_values() {
    assert_eq!(
        sides_from_ipc("duplex").unwrap_err(),
        PRINT_SIDES_UNKNOWN_LABEL
    );
    assert_eq!(sides_from_ipc("").unwrap_err(), PRINT_SIDES_UNKNOWN_LABEL);
}

#[test]
fn copies_bounds_allow_one_to_ninety_nine() {
    assert_eq!(check_copies(1), Ok(()));
    assert_eq!(check_copies(42), Ok(()));
    assert_eq!(check_copies(99), Ok(()));
    assert_eq!(check_copies(0).unwrap_err(), PRINT_COPIES_RANGE_LABEL);
    assert_eq!(check_copies(100).unwrap_err(), PRINT_COPIES_RANGE_LABEL);
}

#[test]
fn message_maps_missing_queue_soft_case() {
    let error = failed(
        "lpstat",
        "lpstat: Invalid destination name in list \"no-such-queue\".",
    );
    assert_eq!(print_message(&error), PRINT_QUEUE_MISSING_LABEL);

    let error = failed("lp", "lp: destination \"no-such-queue\" is not valid.");
    assert_eq!(print_message(&error), PRINT_QUEUE_MISSING_LABEL);
}

#[test]
fn message_maps_missing_cups_tools() {
    let error = PrintError::Io {
        program: "lpstat".to_string(),
        source: io::Error::from(io::ErrorKind::NotFound),
    };
    assert_eq!(print_message(&error), PRINT_CUPS_MISSING_LABEL);
}

#[test]
fn message_falls_back_per_program() {
    assert_eq!(
        print_message(&failed("lp", "lp: unable to print file")),
        PRINT_SEND_FAILED_LABEL
    );
    assert_eq!(
        print_message(&failed("lpstat", "lpstat: boom")),
        PRINT_QUEUE_FAILED_LABEL
    );
    assert_eq!(
        print_message(&failed("cancel", "cancel: cancel-job failed")),
        PRINT_CANCEL_FAILED_LABEL
    );
}

#[test]
fn cancel_accepts_only_configured_queue_jobs() {
    assert!(job_belongs_to_queue(&format!("{QUEUE}-42"), QUEUE));
    assert!(!job_belongs_to_queue("all", QUEUE));
    assert!(!job_belongs_to_queue("", QUEUE));
    assert!(!job_belongs_to_queue("Office_Printer-7", QUEUE));
}
