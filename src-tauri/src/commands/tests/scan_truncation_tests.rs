use std::time::Duration;

use super::escl_server::{EsclServer, Reply};
use super::{
    adf_jpeg_pages, adf_pdf_stack, adf_truncated_pdf, dir_has_no_part_files, saved_file_names,
    unique_temp_dir,
};
use crate::commands::scan_files::document_complete;
use crate::commands::scan_jobs::{TerminalEvent, finish_scan};
use crate::commands::scan_pages::{PagesOutcome, fetch_pages};
use crate::commands::ScanErrorKind;
use crate::escl::{DocFormat, EsclError};

const JOB_ID: &str = "/eSCL/ScanJobs/test-job";
const FETCH_TIMEOUT: Duration = Duration::from_secs(120);

fn truncated_jpeg() -> Vec<u8> {
    let mut page = adf_jpeg_pages()[1].clone();
    let cut = page.len() - 8;
    page.truncate(cut);
    page
}

fn dir_file_count(dir: &std::path::Path) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| entries.filter_map(|entry| entry.ok()).count())
        .unwrap_or(0)
}

#[test]
fn detects_complete_documents() {
    assert!(document_complete(DocFormat::Pdf, &adf_pdf_stack()));
    assert!(document_complete(DocFormat::Jpeg, &adf_jpeg_pages()[0]));

    assert!(document_complete(
        DocFormat::Pdf,
        b"%PDF-1.4 body\n%%EOF\r\n"
    ));
    assert!(document_complete(DocFormat::Pdf, b"%PDF-1.4 body\n%%EOF\n"));
    assert!(document_complete(DocFormat::Pdf, b"%PDF-1.4 body%%EOF"));
    assert!(!document_complete(
        DocFormat::Pdf,
        b"%PDF-1.4 body\n%%EOF\r"
    ));

    assert!(!document_complete(DocFormat::Pdf, &adf_truncated_pdf()));
    assert!(!document_complete(DocFormat::Pdf, b"%PDF-1.4 body"));
    assert!(!document_complete(DocFormat::Jpeg, &truncated_jpeg()));
    assert!(!document_complete(DocFormat::Jpeg, b""));
    assert!(!document_complete(DocFormat::Pdf, b""));
}

#[tokio::test]
async fn canceled_truncated_stack_writes_nothing() {
    let server = EsclServer::spawn(vec![Reply::Page(adf_truncated_pdf()), Reply::Canceled]);
    let client = server.client();
    let job = server.job_url();
    let dir = unique_temp_dir("truncated-cancel");
    let mut counts = Vec::new();

    let outcome = fetch_pages(&client, &job, DocFormat::Pdf, FETCH_TIMEOUT, |pages| {
        counts.push(pages)
    })
    .await;

    let pages = match outcome {
        PagesOutcome::Canceled { pages } => pages,
        other => panic!("expected a canceled outcome, got {other:?}"),
    };
    assert!(pages.is_empty());
    assert!(counts.is_empty());

    let terminal = finish_scan(
        &client,
        &dir,
        JOB_ID,
        &job,
        DocFormat::Pdf,
        PagesOutcome::Canceled { pages },
    )
    .await;

    assert!(matches!(terminal, TerminalEvent::Canceled));
    assert_eq!(dir_file_count(&dir), 0);

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn cancel_after_truncated_page_keeps_complete_pages() {
    let good = adf_jpeg_pages()[0].clone();
    let server = EsclServer::spawn(vec![
        Reply::Page(good.clone()),
        Reply::Page(truncated_jpeg()),
        Reply::Canceled,
    ]);
    let client = server.client();
    let job = server.job_url();
    let dir = unique_temp_dir("truncated-mid-cancel");
    let mut counts = Vec::new();

    let outcome = fetch_pages(&client, &job, DocFormat::Jpeg, FETCH_TIMEOUT, |pages| {
        counts.push(pages)
    })
    .await;

    let pages = match outcome {
        PagesOutcome::Canceled { pages } => pages,
        other => panic!("expected a canceled outcome, got {other:?}"),
    };
    assert_eq!(pages, vec![good.clone()]);
    assert_eq!(counts, vec![1]);

    let terminal = finish_scan(
        &client,
        &dir,
        JOB_ID,
        &job,
        DocFormat::Jpeg,
        PagesOutcome::Canceled { pages },
    )
    .await;

    match terminal {
        TerminalEvent::Done {
            files,
            partial,
            warning,
        } => {
            assert!(partial);
            assert_eq!(warning, None);
            let names = saved_file_names(&files);
            assert_eq!(names.len(), 1);
            assert!(names[0].ends_with("-p1.jpg"), "unexpected name: {names:?}");
            assert_eq!(std::fs::read(&files[0]).expect("read the saved page"), good);
        }
        other => panic!("expected a done terminal, got {other:?}"),
    }
    assert!(dir_has_no_part_files(&dir));

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn truncated_transfer_fails_after_complete_pages() {
    let good = adf_jpeg_pages()[0].clone();
    let server = EsclServer::spawn(vec![
        Reply::Page(good.clone()),
        Reply::Page(truncated_jpeg()),
        Reply::End,
    ]);
    let client = server.client();
    let job = server.job_url();
    let dir = unique_temp_dir("truncated-no-cancel");
    let mut counts = Vec::new();

    let outcome = fetch_pages(&client, &job, DocFormat::Jpeg, FETCH_TIMEOUT, |pages| {
        counts.push(pages)
    })
    .await;

    let (pages, error) = match outcome {
        PagesOutcome::Failed { pages, error } => (pages, error),
        other => panic!("expected a failed outcome, got {other:?}"),
    };
    assert_eq!(pages, vec![good.clone()]);
    assert!(matches!(error, EsclError::Parse(_)));
    assert_eq!(counts, vec![1]);

    let terminal = finish_scan(
        &client,
        &dir,
        JOB_ID,
        &job,
        DocFormat::Jpeg,
        PagesOutcome::Failed { pages, error },
    )
    .await;

    match terminal {
        TerminalEvent::Done {
            files,
            partial,
            warning,
        } => {
            assert!(partial);
            let warning = warning.expect("a failed scan with pages carries a warning");
            assert_eq!(warning.kind, ScanErrorKind::Device);
            assert_eq!(warning.message, "the device returned an unexpected response");
            let names = saved_file_names(&files);
            assert_eq!(names.len(), 1);
            assert!(names[0].ends_with("-p1.jpg"), "unexpected name: {names:?}");
            assert_eq!(std::fs::read(&files[0]).expect("read the saved page"), good);
        }
        other => panic!("expected a done terminal, got {other:?}"),
    }
    assert!(dir_has_no_part_files(&dir));

    let _ = std::fs::remove_dir_all(&dir);
}
