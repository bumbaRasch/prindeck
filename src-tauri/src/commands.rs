use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use chrono::{DateTime, Local};
use reqwest::Url;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;

use crate::config::Config;
use crate::escl::{
    CancelOutcome, DocFormat, EsclClient, EsclError, NextDoc, ScanCaps, ScanParams, ScannerState,
};

pub const EVENT_PROGRESS: &str = "scan-progress";
pub const EVENT_DONE: &str = "scan-done";
pub const EVENT_ERROR: &str = "scan-error";

pub const JOB_PATH_PREFIX: &str = "/eSCL/ScanJobs/";

const FALLBACK_BASE_URL: &str = "http://127.0.0.1:9/eSCL/";

const ADF_NOT_SUPPORTED: &str =
    "the device returned more than one page: the feeder is not supported in this version";
const UNEXPECTED_FORMAT: &str = "the device returned an unexpected format";
const NO_DOCUMENT: &str = "the device returned no document";
const INVALID_JOB_ID: &str = "invalid job id";
const JOB_NOT_FOUND: &str = "job not found";

pub struct AppState {
    pub config: Config,
    pub client: EsclClient,
    caps: Mutex<Option<ScanCaps>>,
    jobs: Mutex<HashMap<String, Url>>,
    startup_error: Option<String>,
}

impl AppState {
    pub fn initialize() -> AppState {
        let (config, client, startup_error) = resolve_state();

        AppState {
            config,
            client,
            caps: Mutex::new(None),
            jobs: Mutex::new(HashMap::new()),
            startup_error,
        }
    }
}

fn resolve_state() -> (Config, EsclClient, Option<String>) {
    let loaded = Config::default_path()
        .map_err(|error| format!("failed to resolve the config path: {error}"))
        .and_then(|path| {
            Config::load_or_create(&path)
                .map_err(|error| format!("failed to load the config {}: {error}", path.display()))
        });

    let mut problems: Vec<String> = Vec::new();

    match loaded {
        Ok(config) => match build_client(&config) {
            Ok(client) => return (config, client, None),
            Err(problem) => problems.push(problem),
        },
        Err(problem) => problems.push(problem),
    }

    let fallback = fallback_config();
    problems.push("default settings are in use".to_string());

    match build_client(&fallback) {
        Ok(client) => (fallback, client, Some(problems.join("; "))),
        Err(problem) => {
            problems.push(problem);
            (fallback, fallback_client(), Some(problems.join("; ")))
        }
    }
}

fn fallback_config() -> Config {
    let home = dirs::home_dir().unwrap_or_else(std::env::temp_dir);
    let documents = dirs::document_dir();

    Config::defaults_for(&home, documents.as_deref())
}

fn build_client(config: &Config) -> Result<EsclClient, String> {
    let url = config.base_url().map_err(|error| error.to_string())?;
    EsclClient::new(url).map_err(|error| error.to_string())
}

fn fallback_client() -> EsclClient {
    let url = Url::parse(FALLBACK_BASE_URL).expect("fallback base URL is a valid literal");
    EsclClient::new(url).expect("HTTP client without TLS cannot fail to build")
}

fn lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>, String> {
    mutex
        .lock()
        .map_err(|_| "internal error: the state is locked after a panic".to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ScanPhase {
    Created,
    Scanning,
    Done,
    Canceled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    pub job_id: String,
    pub phase: ScanPhase,
    pub pages: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanDone {
    pub job_id: String,
    pub file: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanError {
    pub job_id: String,
    pub message: String,
}

fn emit_progress(app: &AppHandle, job_id: &str, phase: ScanPhase, pages: u32) {
    let payload = ScanProgress {
        job_id: job_id.to_string(),
        phase,
        pages,
    };

    if let Err(error) = app.emit(EVENT_PROGRESS, payload) {
        tracing::debug!(%error, "failed to emit scan-progress");
    }
}

fn emit_done(app: &AppHandle, job_id: &str, file: &Path) {
    let payload = ScanDone {
        job_id: job_id.to_string(),
        file: file.display().to_string(),
    };

    if let Err(error) = app.emit(EVENT_DONE, payload) {
        tracing::debug!(%error, "failed to emit scan-done");
    }
}

fn emit_error(app: &AppHandle, job_id: &str, message: String) {
    let payload = ScanError {
        job_id: job_id.to_string(),
        message,
    };

    if let Err(error) = app.emit(EVENT_ERROR, payload) {
        tracing::debug!(%error, "failed to emit scan-error");
    }
}

#[tauri::command]
pub async fn get_capabilities(state: State<'_, AppState>) -> Result<ScanCaps, String> {
    {
        let cached = lock(&state.caps)?;
        if let Some(caps) = cached.as_ref() {
            return Ok(caps.clone());
        }
    }

    let caps = state
        .client
        .capabilities()
        .await
        .map_err(|error| user_message(&error))?;

    {
        let mut cached = lock(&state.caps)?;
        *cached = Some(caps.clone());
    }

    Ok(caps)
}

#[tauri::command]
pub async fn start_scan(
    app: AppHandle,
    state: State<'_, AppState>,
    params: ScanParams,
) -> Result<String, String> {
    let status = state.client.status().await.map_err(|error| {
        tracing::warn!(%error, "scanner status request failed");
        user_message(&error)
    })?;

    if status.state != ScannerState::Idle {
        return Err("the device is busy".to_string());
    }

    let job = state.client.start(&params).await.map_err(|error| {
        tracing::warn!(%error, "scan start request failed");
        user_message(&error)
    })?;
    let job_id = job.path().to_string();

    let duplicate = {
        let mut jobs = lock(&state.jobs)?;
        let duplicate = jobs.contains_key(&job_id);
        if !duplicate {
            jobs.insert(job_id.clone(), job.clone());
        }
        duplicate
    };

    if duplicate {
        best_effort_cancel(&state.client, &job).await;
        return Err("this job is already running".to_string());
    }

    tracing::info!(
        %job_id,
        source = ?params.source,
        dpi = params.dpi,
        color = ?params.color,
        format = ?params.format,
        "scan started"
    );

    emit_progress(&app, &job_id, ScanPhase::Created, 0);

    let client = state.client.clone();
    let scan_dir = state.config.scan_dir.clone();
    let format = params.format;
    let task_job_id = job_id.clone();

    tauri::async_runtime::spawn(run_scan(app, client, scan_dir, task_job_id, job, format));

    Ok(job_id)
}

#[tauri::command]
pub async fn cancel_scan(state: State<'_, AppState>, job_id: String) -> Result<(), String> {
    let job = {
        validate_job_id(&job_id)?;
        let jobs = lock(&state.jobs)?;
        jobs.get(&job_id)
            .cloned()
            .ok_or_else(|| JOB_NOT_FOUND.to_string())?
    };

    match state.client.cancel(&job).await {
        Ok(CancelOutcome::Canceled | CancelOutcome::AlreadyFinished) => Ok(()),
        Err(error) => {
            tracing::warn!(%error, %job_id, "scan cancel request failed");
            Err(user_message(&error))
        }
    }
}

#[tauri::command]
pub fn open_scan_folder(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    state
        .config
        .ensure_scan_dir()
        .map_err(|error| error.to_string())?;

    let path = state.config.scan_dir.display().to_string();

    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|error| format!("failed to open the scan folder: {error}"))
}

#[tauri::command]
pub fn get_startup_error(state: State<'_, AppState>) -> Option<String> {
    state.startup_error.clone()
}

pub fn user_message(error: &EsclError) -> String {
    match error {
        EsclError::Http { status: 503, .. } => "the device is busy".to_string(),
        EsclError::Http { status: 409, .. } => {
            "the device does not support this combination of settings".to_string()
        }
        EsclError::Http { status: 500, body } if mentions_feeder(body) => {
            "the feeder is out of paper".to_string()
        }
        EsclError::Timeout => "the device is not responding".to_string(),
        EsclError::Transport(_) => "no connection to the device".to_string(),
        EsclError::InvalidParams(message) => message.clone(),
        EsclError::Http { status, .. } => format!("the device returned HTTP {status}"),
        EsclError::Parse(_) => "the device returned an unexpected response".to_string(),
        EsclError::InvalidUrl(_) => "invalid device address".to_string(),
    }
}

fn mentions_feeder(body: &str) -> bool {
    let body = body.to_ascii_lowercase();

    ["feeder", "adf", "no document", "paper"]
        .iter()
        .any(|needle| body.contains(needle))
}

pub fn validate_job_id(job_id: &str) -> Result<(), String> {
    let Some(rest) = job_id.strip_prefix(JOB_PATH_PREFIX) else {
        return Err(INVALID_JOB_ID.to_string());
    };

    let valid = !rest.is_empty()
        && rest.len() <= 128
        && rest
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_');

    if valid {
        Ok(())
    } else {
        Err(INVALID_JOB_ID.to_string())
    }
}

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

fn part_path(dir: &Path, job_id: &str) -> PathBuf {
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

fn remove_part(part: &Path) {
    if let Err(error) = fs::remove_file(part) {
        if error.kind() != std::io::ErrorKind::NotFound {
            tracing::debug!(%error, path = %part.display(), "failed to remove the temporary file");
        }
    }
}

enum ScanOutcome {
    Done(PathBuf),
    Canceled,
    Failed(String),
}

async fn run_scan(
    app: AppHandle,
    client: EsclClient,
    scan_dir: PathBuf,
    job_id: String,
    job: Url,
    format: DocFormat,
) {
    emit_progress(&app, &job_id, ScanPhase::Scanning, 0);

    let outcome = fetch_document(&client, &scan_dir, &job, &job_id, format).await;

    match outcome {
        ScanOutcome::Done(file) => {
            tracing::info!(%job_id, path = %file.display(), "scan saved");
            emit_done(&app, &job_id, &file);
            emit_progress(&app, &job_id, ScanPhase::Done, 1);
        }
        ScanOutcome::Canceled => {
            tracing::info!(%job_id, "scan canceled");
            emit_progress(&app, &job_id, ScanPhase::Canceled, 0);
        }
        ScanOutcome::Failed(message) => {
            tracing::warn!(%job_id, %message, "scan failed");
            emit_error(&app, &job_id, message);
        }
    }

    forget_job(&app, &job_id);
}

async fn fetch_document(
    client: &EsclClient,
    scan_dir: &Path,
    job: &Url,
    job_id: &str,
    format: DocFormat,
) -> ScanOutcome {
    let part = part_path(scan_dir, job_id);
    let mut written = false;

    loop {
        match client.next_document(job).await {
            Ok(NextDoc::Document(bytes)) => {
                if written {
                    best_effort_cancel(client, job).await;
                    remove_part(&part);
                    return ScanOutcome::Failed(ADF_NOT_SUPPORTED.to_string());
                }

                if !format_matches(format, &bytes) {
                    best_effort_cancel(client, job).await;
                    remove_part(&part);
                    return ScanOutcome::Failed(UNEXPECTED_FORMAT.to_string());
                }

                if let Err(error) = fs::write(&part, &bytes) {
                    remove_part(&part);
                    best_effort_cancel(client, job).await;
                    return ScanOutcome::Failed(format!("failed to save the file: {error}"));
                }

                written = true;
            }
            Ok(NextDoc::End) => {
                if !written {
                    remove_part(&part);
                    return ScanOutcome::Failed(NO_DOCUMENT.to_string());
                }

                let target = unique_scan_path(scan_dir, Local::now(), format);
                if let Err(error) = fs::rename(&part, &target) {
                    remove_part(&part);
                    return ScanOutcome::Failed(format!("failed to save the file: {error}"));
                }

                return ScanOutcome::Done(target);
            }
            Ok(NextDoc::Canceled) => {
                remove_part(&part);
                return ScanOutcome::Canceled;
            }
            Err(error) => {
                remove_part(&part);
                best_effort_cancel(client, job).await;
                return ScanOutcome::Failed(user_message(&error));
            }
        }
    }
}

async fn best_effort_cancel(client: &EsclClient, job: &Url) {
    if let Err(error) = client.cancel(job).await {
        tracing::debug!(%error, "failed to cancel the job after an error");
    }
}

fn forget_job(app: &AppHandle, job_id: &str) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };

    match state.jobs.lock() {
        Ok(mut jobs) => {
            jobs.remove(job_id);
        }
        Err(error) => tracing::error!(%error, "failed to update the job registry"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json::json;

    fn sample_error(status: u16, body: &str) -> EsclError {
        EsclError::Http {
            status,
            body: body.to_string(),
        }
    }

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
    fn user_message_maps_device_errors() {
        assert_eq!(user_message(&sample_error(503, "")), "the device is busy");
        assert_eq!(
            user_message(&sample_error(409, "")),
            "the device does not support this combination of settings"
        );
        assert_eq!(
            user_message(&sample_error(500, "<e>Feeder empty</e>")),
            "the feeder is out of paper"
        );
        assert_eq!(
            user_message(&EsclError::Timeout),
            "the device is not responding"
        );
        assert_eq!(
            user_message(&EsclError::Transport("connect refused".to_string())),
            "no connection to the device"
        );
        assert_eq!(
            user_message(&EsclError::InvalidParams(
                "the device does not support 150 dpi".to_string()
            )),
            "the device does not support 150 dpi"
        );
    }

    #[test]
    fn user_message_keeps_http_code_for_other_statuses() {
        assert_eq!(
            user_message(&sample_error(418, "")),
            "the device returned HTTP 418"
        );
        assert_eq!(
            user_message(&sample_error(500, "<e>internal</e>")),
            "the device returned HTTP 500"
        );
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
    fn serializes_done_and_error_payloads() {
        let job_id = "/eSCL/ScanJobs/abc";

        let done = serde_json::to_value(ScanDone {
            job_id: job_id.to_string(),
            file: "/tmp/scans/scan-20261027-141530.pdf".to_string(),
        })
        .expect("done serializes");
        assert_eq!(
            done,
            json!({
                "jobId": job_id,
                "file": "/tmp/scans/scan-20261027-141530.pdf"
            })
        );

        let error = serde_json::to_value(ScanError {
            job_id: job_id.to_string(),
            message: "the device is busy".to_string(),
        })
        .expect("error serializes");
        assert_eq!(
            error,
            json!({ "jobId": job_id, "message": "the device is busy" })
        );
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

    #[test]
    fn fallback_base_url_is_valid() {
        let url = Url::parse(FALLBACK_BASE_URL).expect("fallback URL literal is valid");

        assert!(EsclClient::new(url).is_ok());
    }
}
