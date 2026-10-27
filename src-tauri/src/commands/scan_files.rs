use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};

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

pub fn unique_scan_path(dir: &Path, timestamp: DateTime<Local>, format: DocFormat) -> PathBuf {
    let base = scan_file_name(timestamp, format);
    let first = dir.join(&base);

    if !first.exists() {
        return first;
    }

    let stem = base
        .rsplit_once('.')
        .map(|(stem, _)| stem.to_string())
        .unwrap_or_else(|| base.clone());
    let extension = extension_for(format);

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
