use std::time::{Duration, Instant};

use reqwest::Url;

use super::scan_files::format_matches;
use crate::escl::{DocFormat, EsclClient, EsclError, NextDoc};

const MAX_PAUSE: Duration = Duration::from_secs(1);

#[derive(Debug)]
pub enum PagesOutcome {
    Complete { pages: Vec<Vec<u8>> },
    Canceled { pages: Vec<Vec<u8>> },
    Failed { pages: Vec<Vec<u8>>, error: EsclError },
}

pub fn next_delay(prev_request: Duration) -> Duration {
    (prev_request / 2).min(MAX_PAUSE)
}

pub(super) async fn fetch_pages(
    client: &EsclClient,
    job: &Url,
    format: DocFormat,
    mut on_page: impl FnMut(u32),
) -> PagesOutcome {
    let mut pages: Vec<Vec<u8>> = Vec::new();

    loop {
        let started = Instant::now();
        let attempt = client.next_document(job).await;
        let elapsed = started.elapsed();

        match attempt {
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

        tokio::time::sleep(next_delay(elapsed)).await;
    }
}
