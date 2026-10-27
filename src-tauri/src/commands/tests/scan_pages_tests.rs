use std::time::Duration;

use super::escl_server::{EsclServer, Reply};
use super::{
    adf_pdf_stack, dir_has_no_part_files, jpeg_page, pdf_page, sample_time, unique_temp_dir,
};
use crate::commands::scan_files::write_pages;
use crate::commands::scan_jobs::{TerminalEvent, finish_scan};
use crate::commands::scan_pages::{PagesOutcome, fetch_pages};
use crate::escl::{DocFormat, EsclError};

const JOB_ID: &str = "/eSCL/ScanJobs/test-job";
const FETCH_TIMEOUT: Duration = Duration::from_secs(120);

#[tokio::test]
async fn complete_outcome_collects_all_pages() {
    let server = EsclServer::spawn(vec![
        Reply::Page(jpeg_page(1)),
        Reply::Page(jpeg_page(2)),
        Reply::Page(jpeg_page(3)),
        Reply::End,
    ]);
    let client = server.client();
    let job = server.job_url();
    let mut counts = Vec::new();

    let outcome = fetch_pages(&client, &job, DocFormat::Jpeg, FETCH_TIMEOUT, |pages| {
        counts.push(pages)
    })
    .await;

    match outcome {
        PagesOutcome::Complete { pages } => {
            assert_eq!(pages, vec![jpeg_page(1), jpeg_page(2), jpeg_page(3)]);
        }
        other => panic!("expected a complete outcome, got {other:?}"),
    }
    assert_eq!(counts, vec![1, 2, 3]);
}

#[tokio::test]
async fn pdf_stack_arrives_as_one_document() {
    let stack = adf_pdf_stack();
    let server = EsclServer::spawn(vec![Reply::Page(stack.clone()), Reply::End]);
    let client = server.client();
    let job = server.job_url();
    let dir = unique_temp_dir("pdf-stack");
    let mut counts = Vec::new();

    let outcome = fetch_pages(&client, &job, DocFormat::Pdf, FETCH_TIMEOUT, |pages| {
        counts.push(pages)
    })
    .await;

    let pages = match outcome {
        PagesOutcome::Complete { pages } => pages,
        other => panic!("expected a complete outcome, got {other:?}"),
    };
    assert_eq!(pages, vec![stack.clone()]);
    assert_eq!(counts, vec![1]);

    let (files, storage) = write_pages(&dir, JOB_ID, DocFormat::Pdf, &pages, false, sample_time());
    assert!(storage.is_none());
    assert_eq!(files.len(), 1);
    assert_eq!(
        files[0].file_name().and_then(|name| name.to_str()),
        Some("scan-20261027-141530.pdf")
    );
    assert_eq!(
        std::fs::read(&files[0]).expect("read the saved stack"),
        stack
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn canceled_keeps_pages() {
    let server = EsclServer::spawn(vec![
        Reply::Page(jpeg_page(1)),
        Reply::Page(jpeg_page(2)),
        Reply::Canceled,
    ]);
    let client = server.client();
    let job = server.job_url();
    let dir = unique_temp_dir("cancel-keep");

    let outcome = fetch_pages(&client, &job, DocFormat::Jpeg, FETCH_TIMEOUT, |_| {}).await;

    let pages = match outcome {
        PagesOutcome::Canceled { pages } => pages,
        other => panic!("expected a canceled outcome, got {other:?}"),
    };
    assert_eq!(pages.len(), 2);

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
            assert_eq!(files.len(), 2);
            assert!(files.iter().all(|file| file.exists()));
        }
        other => panic!("expected a done terminal, got {other:?}"),
    }
    assert!(
        server
            .request_lines()
            .iter()
            .any(|line| line.starts_with("DELETE /eSCL/ScanJobs/test-job "))
    );
    assert!(dir_has_no_part_files(&dir));

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn empty_stack_is_a_device_error() {
    let server = EsclServer::spawn(vec![Reply::End]);
    let client = server.client();
    let job = server.job_url();

    let outcome = fetch_pages(&client, &job, DocFormat::Pdf, FETCH_TIMEOUT, |_| {}).await;

    match outcome {
        PagesOutcome::Failed { pages, error } => {
            assert!(pages.is_empty());
            assert!(matches!(error, EsclError::Parse(_)));
        }
        other => panic!("expected a failed outcome, got {other:?}"),
    }
}

#[tokio::test]
async fn failed_fetch_keeps_pages() {
    let server = EsclServer::spawn(vec![Reply::Page(pdf_page(1)), Reply::Failure]);
    let client = server.client();
    let job = server.job_url();

    let outcome = fetch_pages(&client, &job, DocFormat::Pdf, FETCH_TIMEOUT, |_| {}).await;

    match outcome {
        PagesOutcome::Failed { pages, error } => {
            assert_eq!(pages, vec![pdf_page(1)]);
            assert!(matches!(error, EsclError::Http { status: 500, .. }));
        }
        other => panic!("expected a failed outcome, got {other:?}"),
    }
}
