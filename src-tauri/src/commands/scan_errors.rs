use serde::Serialize;

use crate::escl::EsclError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ScanErrorKind {
    Transport,
    Busy,
    Invalid,
    Device,
    Storage,
    Canceled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanFailure {
    pub kind: ScanErrorKind,
    pub message: String,
}

impl ScanFailure {
    pub fn new(kind: ScanErrorKind, message: impl Into<String>) -> ScanFailure {
        ScanFailure {
            kind,
            message: message.into(),
        }
    }

    pub fn from_escl(error: &EsclError) -> ScanFailure {
        ScanFailure::new(error_kind(error), user_message(error))
    }
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

fn error_kind(error: &EsclError) -> ScanErrorKind {
    match error {
        EsclError::Timeout | EsclError::Transport(_) => ScanErrorKind::Transport,
        EsclError::InvalidParams(_) | EsclError::InvalidUrl(_) => ScanErrorKind::Invalid,
        EsclError::Http { status: 503, .. } => ScanErrorKind::Busy,
        EsclError::Http { status: 409, .. } => ScanErrorKind::Invalid,
        EsclError::Http { .. } | EsclError::Parse(_) => ScanErrorKind::Device,
    }
}

fn mentions_feeder(body: &str) -> bool {
    let body = body.to_ascii_lowercase();

    ["feeder", "adf", "no document", "paper"]
        .iter()
        .any(|needle| body.contains(needle))
}
