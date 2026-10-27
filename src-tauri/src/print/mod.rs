mod lp;

#[cfg(test)]
mod tests;

use serde::Serialize;

pub use lp::{PrintError, cancel, print_file, queue_jobs};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sides {
    OneSided,
    LongEdge,
    ShortEdge,
}

impl Sides {
    pub fn as_lp_option(&self) -> &'static str {
        match self {
            Sides::OneSided => "one-sided",
            Sides::LongEdge => "two-sided-long-edge",
            Sides::ShortEdge => "two-sided-short-edge",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrintOptions {
    pub copies: u32,
    pub sides: Sides,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CupsJob {
    pub id: String,
    pub owner: String,
    pub size_kb: u64,
}
