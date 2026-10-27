use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};

use super::{ScanErrorKind, ScanFailure};
use crate::escl::DocFormat;

pub fn format_matches(format: DocFormat, bytes: &[u8]) -> bool {
    match format {
        DocFormat::Pdf => bytes.starts_with(b"%PDF-"),
        DocFormat::Jpeg => bytes.starts_with(&[0xFF, 0xD8, 0xFF]),
    }
}

pub fn extension_for(format: DocFormat) -> &'static str {
    match format {
        DocFormat::Pdf => "pdf",
        DocFormat::Jpeg => "jpg",
    }
}

pub fn scan_file_name(timestamp: DateTime<Local>, format: DocFormat) -> String {
    format!(
        "scan-{}.{}",
        timestamp.format("%Y%m%d-%H%M%S"),
        extension_for(format)
    )
}

pub fn page_file_name(timestamp: DateTime<Local>, format: DocFormat, page: u32) -> String {
    format!(
        "scan-{}-p{page}.{}",
        timestamp.format("%Y%m%d-%H%M%S"),
        extension_for(format)
    )
}

pub fn partial_file_name(timestamp: DateTime<Local>, format: DocFormat) -> String {
    format!(
        "scan-{}-partial.{}",
        timestamp.format("%Y%m%d-%H%M%S"),
        extension_for(format)
    )
}

pub(super) fn page_file_names(
    timestamp: DateTime<Local>,
    format: DocFormat,
    page_count: usize,
    partial: bool,
) -> Vec<String> {
    if format == DocFormat::Pdf {
        let base = if partial {
            partial_file_name(timestamp, format)
        } else {
            scan_file_name(timestamp, format)
        };
        return vec![base; page_count];
    }

    (1..=page_count)
        .map(|page| page_file_name(timestamp, format, page as u32))
        .collect()
}

pub fn unique_path(dir: &Path, base: &str) -> PathBuf {
    let first = dir.join(base);

    if !first.exists() {
        return first;
    }

    let (stem, extension) = base
        .rsplit_once('.')
        .map(|(stem, extension)| (stem.to_string(), extension.to_string()))
        .unwrap_or_else(|| (base.to_string(), String::new()));

    for index in 2..=9999u32 {
        let candidate = dir.join(format!("{stem}-{index}.{extension}"));
        if !candidate.exists() {
            return candidate;
        }
    }

    dir.join(format!("{stem}-{}.{extension}", std::process::id()))
}

pub(super) fn part_path(dir: &Path, job_id: &str) -> PathBuf {
    let tail = job_id.rsplit('/').next().unwrap_or("job");
    let safe: String = tail
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-')
        .collect();
    let name = if safe.is_empty() {
        "job".to_string()
    } else {
        safe
    };

    dir.join(format!(".scan-{name}.part"))
}

pub(super) fn remove_part(part: &Path) {
    if let Err(error) = fs::remove_file(part)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        tracing::debug!(%error, path = %part.display(), "failed to remove the temporary file");
    }
}

pub(super) fn write_pages(
    scan_dir: &Path,
    job_id: &str,
    format: DocFormat,
    pages: &[Vec<u8>],
    partial: bool,
    timestamp: DateTime<Local>,
) -> (Vec<PathBuf>, Option<ScanFailure>) {
    let names = page_file_names(timestamp, format, pages.len(), partial);
    let part = part_path(scan_dir, job_id);
    let mut files = Vec::new();

    for (bytes, base) in pages.iter().zip(names) {
        let target = unique_path(scan_dir, &base);

        if let Err(error) = fs::write(&part, bytes) {
            remove_part(&part);
            return (files, Some(storage_failure(&error)));
        }

        if let Err(error) = fs::rename(&part, &target) {
            remove_part(&part);
            return (files, Some(storage_failure(&error)));
        }

        files.push(target);
    }

    (files, None)
}

fn storage_failure(error: &std::io::Error) -> ScanFailure {
    ScanFailure::new(
        ScanErrorKind::Storage,
        format!("failed to save the file: {error}"),
    )
}
