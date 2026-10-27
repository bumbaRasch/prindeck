use std::fs;

use super::{dir_has_no_part_files, jpeg_page, sample_time, unique_temp_dir};
use crate::commands::scan_files::{page_file_names, part_path, unique_path, write_pages};
use crate::commands::ScanErrorKind;
use crate::escl::DocFormat;

const JOB_ID: &str = "/eSCL/ScanJobs/job-a";

#[test]
fn jpeg_stack_names_pages() {
    let names = page_file_names(sample_time(), DocFormat::Jpeg, 3, false);

    assert_eq!(
        names,
        vec![
            "scan-20261027-141530-p1.jpg",
            "scan-20261027-141530-p2.jpg",
            "scan-20261027-141530-p3.jpg",
        ]
    );
}

#[test]
fn canceled_pdf_names_partial() {
    let names = page_file_names(sample_time(), DocFormat::Pdf, 1, true);

    assert_eq!(names, vec!["scan-20261027-141530-partial.pdf"]);
}

#[test]
fn complete_single_pdf_keeps_base_name() {
    let names = page_file_names(sample_time(), DocFormat::Pdf, 1, false);

    assert_eq!(names, vec!["scan-20261027-141530.pdf"]);
}

#[test]
fn multi_page_pdf_names_pages_until_merge() {
    let names = page_file_names(sample_time(), DocFormat::Pdf, 2, true);

    assert_eq!(
        names,
        vec!["scan-20261027-141530-p1.pdf", "scan-20261027-141530-p2.pdf"]
    );
}

#[test]
fn empty_stacks_have_no_names() {
    assert!(page_file_names(sample_time(), DocFormat::Jpeg, 0, true).is_empty());
    assert!(page_file_names(sample_time(), DocFormat::Pdf, 0, false).is_empty());
}

#[test]
fn partial_names_avoid_collisions() {
    let dir = unique_temp_dir("partial-collision");
    let base = "scan-20261027-141530-partial.pdf";

    let first = unique_path(&dir, base);
    assert_eq!(
        first.file_name().and_then(|name| name.to_str()),
        Some(base)
    );

    fs::write(&first, b"%PDF-1.4").expect("write the first file");

    let second = unique_path(&dir, base);
    assert_eq!(
        second.file_name().and_then(|name| name.to_str()),
        Some("scan-20261027-141530-partial-2.pdf")
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn builds_part_path_from_job_id() {
    let part = part_path(std::path::Path::new("/tmp/scans"), "/eSCL/ScanJobs/abc-123");

    assert_eq!(
        part.file_name().and_then(|name| name.to_str()),
        Some(".scan-abc-123.part")
    );

    let weird = part_path(std::path::Path::new("/tmp/scans"), "/eSCL/ScanJobs/../../eth");
    assert_eq!(
        weird.file_name().and_then(|name| name.to_str()),
        Some(".scan-eth.part")
    );
}

#[test]
fn write_pages_writes_unique_page_files() {
    let dir = unique_temp_dir("write-pages");
    fs::write(dir.join("scan-20261027-141530-p1.jpg"), jpeg_page(9))
        .expect("seed an existing page file");

    let (files, storage) = write_pages(
        &dir,
        JOB_ID,
        DocFormat::Jpeg,
        &[jpeg_page(1), jpeg_page(2)],
        false,
        sample_time(),
    );

    assert!(storage.is_none());
    let names: Vec<String> = files
        .iter()
        .map(|file| {
            file.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_string()
        })
        .collect();
    assert_eq!(
        names,
        vec!["scan-20261027-141530-p1-2.jpg", "scan-20261027-141530-p2.jpg"]
    );
    assert_eq!(
        fs::read(&files[0]).expect("read the first page"),
        jpeg_page(1)
    );
    assert_eq!(
        fs::read(&files[1]).expect("read the second page"),
        jpeg_page(2)
    );
    assert!(dir_has_no_part_files(&dir));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn write_pages_maps_io_errors_to_storage_failure() {
    let blocker = unique_temp_dir("write-block");
    let block_file = blocker.join("not-a-dir");
    fs::write(&block_file, b"").expect("write the blocking file");

    let (files, storage) = write_pages(
        &block_file,
        JOB_ID,
        DocFormat::Jpeg,
        &[jpeg_page(1)],
        false,
        sample_time(),
    );

    assert!(files.is_empty());
    let Some(failure) = storage else {
        panic!("expected a storage failure");
    };
    assert_eq!(failure.kind, ScanErrorKind::Storage);

    let _ = fs::remove_dir_all(&blocker);
}
