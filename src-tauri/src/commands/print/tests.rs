use std::io;
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

use super::{
    PrintIpcError, cancel_job_id, check_copies, job_belongs_to_queue, print_message, sides_from_ipc,
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
        sides_from_ipc("bogus").unwrap_err(),
        PrintIpcError::SidesUnknown
    );
    assert_eq!(sides_from_ipc("").unwrap_err(), PrintIpcError::SidesUnknown);
}

#[test]
fn copies_bounds_allow_one_to_ninety_nine() {
    assert_eq!(check_copies(1), Ok(()));
    assert_eq!(check_copies(42), Ok(()));
    assert_eq!(check_copies(99), Ok(()));
    assert_eq!(check_copies(0).unwrap_err(), PrintIpcError::CopiesRange);
    assert_eq!(check_copies(100).unwrap_err(), PrintIpcError::CopiesRange);
}

#[test]
fn message_maps_missing_queue_soft_case() {
    let error = failed(
        "lpstat",
        "lpstat: Invalid destination name in list \"no-such-queue\".",
    );
    assert_eq!(print_message(&error), PrintIpcError::QueueMissing);

    let error = failed("lp", "lp: destination \"no-such-queue\" is not valid.");
    assert_eq!(print_message(&error), PrintIpcError::QueueMissing);
}

#[test]
fn message_maps_missing_cups_tools() {
    let error = PrintError::Io {
        program: "lpstat".to_string(),
        source: io::Error::from(io::ErrorKind::NotFound),
    };
    assert_eq!(print_message(&error), PrintIpcError::CupsMissing);
}

#[test]
fn message_falls_back_per_program() {
    assert_eq!(
        print_message(&failed("lp", "lp: unable to print file")),
        PrintIpcError::SendFailed
    );
    assert_eq!(
        print_message(&failed("lpstat", "lpstat: boom")),
        PrintIpcError::QueueFailed
    );
    assert_eq!(
        print_message(&failed("cancel", "cancel: cancel-job failed")),
        PrintIpcError::CancelFailed
    );
}

#[test]
fn message_maps_missing_job_id_to_cancel_failed() {
    assert_eq!(
        print_message(&PrintError::MissingJobId),
        PrintIpcError::CancelFailed
    );
}

#[test]
fn cancel_accepts_only_configured_queue_jobs() {
    assert_eq!(
        cancel_job_id(&format!("{QUEUE}-42"), QUEUE),
        Ok(format!("{QUEUE}-42"))
    );
    assert_eq!(
        cancel_job_id("all", QUEUE).unwrap_err(),
        PrintIpcError::JobOutsideQueue
    );
    assert_eq!(
        cancel_job_id("", QUEUE).unwrap_err(),
        PrintIpcError::JobOutsideQueue
    );
    assert_eq!(
        cancel_job_id("Office_Printer-7", QUEUE).unwrap_err(),
        PrintIpcError::JobOutsideQueue
    );
    assert!(!job_belongs_to_queue("Office_Printer-7", QUEUE));
}

#[test]
fn ipc_errors_serialize_to_kebab_case_kinds() {
    let cases = [
        (PrintIpcError::CopiesRange, r#"{"kind":"copies-range"}"#),
        (PrintIpcError::SidesUnknown, r#"{"kind":"sides-unknown"}"#),
        (
            PrintIpcError::JobOutsideQueue,
            r#"{"kind":"job-outside-queue"}"#,
        ),
        (PrintIpcError::CupsMissing, r#"{"kind":"cups-missing"}"#),
        (PrintIpcError::QueueMissing, r#"{"kind":"queue-missing"}"#),
        (PrintIpcError::SendFailed, r#"{"kind":"send-failed"}"#),
        (PrintIpcError::QueueFailed, r#"{"kind":"queue-failed"}"#),
        (PrintIpcError::CancelFailed, r#"{"kind":"cancel-failed"}"#),
    ];

    for (error, expected) in cases {
        assert_eq!(serde_json::to_string(&error).unwrap(), expected);
    }
}
