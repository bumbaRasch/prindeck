mod caps_tests;
mod error_tests;
mod escl_server;
mod scan_files_tests;
mod scan_pages_tests;
mod scan_tests;
mod terminal_tests;

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, Local, TimeZone};

fn sample_time() -> DateTime<Local> {
    Local
        .with_ymd_and_hms(2026, 10, 27, 14, 15, 30)
        .single()
        .expect("valid local time")
}

fn unique_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!(
        "prindeck-commands-{name}-{}-{nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn dir_has_no_part_files(dir: &Path) -> bool {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .all(|entry| !entry.file_name().to_string_lossy().contains(".part"))
        })
        .unwrap_or(false)
}

fn jpeg_page(marker: u8) -> Vec<u8> {
    vec![0xFF, 0xD8, 0xFF, 0xE0, marker]
}

fn pdf_page(marker: u8) -> Vec<u8> {
    format!("%PDF-1.4 page {marker}").into_bytes()
}

fn adf_jpeg_pages() -> Vec<Vec<u8>> {
    vec![
        include_bytes!("../../../../fixtures/escl/adf/adf-page-1.jpg").to_vec(),
        include_bytes!("../../../../fixtures/escl/adf/adf-page-2.jpg").to_vec(),
        include_bytes!("../../../../fixtures/escl/adf/adf-page-3.jpg").to_vec(),
    ]
}

fn adf_pdf_stack() -> Vec<u8> {
    include_bytes!("../../../../fixtures/escl/adf/adf-2sheets-300-gray.pdf").to_vec()
}

fn saved_file_names(files: &[std::path::PathBuf]) -> Vec<String> {
    files
        .iter()
        .map(|file| {
            file.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_string()
        })
        .collect()
}
