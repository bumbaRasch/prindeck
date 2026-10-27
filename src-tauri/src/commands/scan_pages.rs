use std::time::Duration;

use reqwest::Url;

use super::scan_files::format_matches;
use crate::escl::{DocFormat, EsclClient, EsclError, NextDoc};

#[derive(Debug)]
pub enum PagesOutcome {
    Complete {
        pages: Vec<Vec<u8>>,
    },
    Canceled {
        pages: Vec<Vec<u8>>,
    },
    Failed {
        pages: Vec<Vec<u8>>,
        error: EsclError,
    },
}

pub(super) async fn fetch_pages(
    client: &EsclClient,
    job: &Url,
    format: DocFormat,
    document_timeout: Duration,
    mut on_page: impl FnMut(u32),
) -> PagesOutcome {
    let mut pages: Vec<Vec<u8>> = Vec::new();

    loop {
        match client.next_document(job, document_timeout).await {
            Ok(NextDoc::Document(bytes)) => {
                if !format_matches(format, &bytes) {
                    return PagesOutcome::Failed {
                        pages,
                        error: EsclError::Parse("unexpected document format".to_string()),
                    };
                }
                pages.push(bytes);
                on_page(pages.len() as u32);
            }
            Ok(NextDoc::End) => {
                if pages.is_empty() {
                    return PagesOutcome::Failed {
                        pages,
                        error: EsclError::Parse("the device returned no document".to_string()),
                    };
                }
                return PagesOutcome::Complete { pages };
            }
            Ok(NextDoc::Canceled) => return PagesOutcome::Canceled { pages },
            Err(error) => return PagesOutcome::Failed { pages, error },
        }
    }
}
