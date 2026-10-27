use std::path::PathBuf;

use super::{jpeg_page, pdf_page};
use crate::commands::scan_jobs::{TerminalEvent, terminal_for};
use crate::commands::scan_pages::PagesOutcome;
use crate::commands::{ScanErrorKind, ScanFailure};
use crate::escl::EsclError;

fn written_files() -> Vec<PathBuf> {
    vec![PathBuf::from("/tmp/scans/scan-20261027-141530-p1.jpg")]
}

#[test]
fn terminal_event_complete_is_done_full() {
    let outcome = PagesOutcome::Complete {
        pages: vec![jpeg_page(1)],
    };

    match terminal_for(&outcome, &written_files(), None) {
        TerminalEvent::Done {
            files,
            partial,
            warning,
        } => {
            assert!(!partial);
            assert_eq!(warning, None);
            assert_eq!(files, written_files());
        }
        other => panic!("expected a done terminal, got {other:?}"),
    }
}

#[test]
fn terminal_event_failed_with_pages_is_done_partial() {
    let outcome = PagesOutcome::Failed {
        pages: vec![jpeg_page(1)],
        error: EsclError::Timeout,
    };

    match terminal_for(&outcome, &written_files(), None) {
        TerminalEvent::Done {
            files,
            partial,
            warning,
        } => {
            assert!(partial);
            assert_eq!(warning.as_deref(), Some("the device is not responding"));
            assert_eq!(files, written_files());
        }
        other => panic!("expected a done terminal, got {other:?}"),
    }
}

#[test]
fn terminal_event_failed_empty_is_error() {
    let outcome = PagesOutcome::Failed {
        pages: Vec::new(),
        error: EsclError::Timeout,
    };

    match terminal_for(&outcome, &[], None) {
        TerminalEvent::Error { failure } => {
            assert_eq!(failure.kind, ScanErrorKind::Transport);
            assert_eq!(failure.message, "the device is not responding");
        }
        other => panic!("expected an error terminal, got {other:?}"),
    }
}

#[test]
fn terminal_event_canceled_with_pages_is_done_partial() {
    let outcome = PagesOutcome::Canceled {
        pages: vec![jpeg_page(1)],
    };

    match terminal_for(&outcome, &written_files(), None) {
        TerminalEvent::Done {
            files,
            partial,
            warning,
        } => {
            assert!(partial);
            assert_eq!(warning, None);
            assert_eq!(files, written_files());
        }
        other => panic!("expected a done terminal, got {other:?}"),
    }
}

#[test]
fn terminal_event_canceled_empty_is_progress_canceled_only() {
    let outcome = PagesOutcome::Canceled { pages: Vec::new() };

    let terminal = terminal_for(&outcome, &[], None);

    assert!(matches!(terminal, TerminalEvent::Canceled));
}

#[test]
fn storage_failure_over_files_is_done_partial() {
    let outcome = PagesOutcome::Complete {
        pages: vec![pdf_page(1)],
    };
    let failure = ScanFailure::new(ScanErrorKind::Storage, "failed to save the file: full");

    match terminal_for(&outcome, &written_files(), Some(failure)) {
        TerminalEvent::Done {
            partial, warning, ..
        } => {
            assert!(partial);
            assert_eq!(warning.as_deref(), Some("failed to save the file: full"));
        }
        other => panic!("expected a done terminal, got {other:?}"),
    }
}

#[test]
fn storage_failure_without_files_is_error() {
    let outcome = PagesOutcome::Complete {
        pages: vec![pdf_page(1)],
    };
    let failure = ScanFailure::new(ScanErrorKind::Storage, "failed to save the file: full");

    match terminal_for(&outcome, &[], Some(failure)) {
        TerminalEvent::Error { failure } => assert_eq!(failure.kind, ScanErrorKind::Storage),
        other => panic!("expected an error terminal, got {other:?}"),
    }
}
