//! crates.io connector client.

use async_trait::async_trait;
use ferris_core::Source;
use ferris_ingest::{ConnectorError, HttpFetcher, NormalizedOutput, SourceConnector};

const TOP_URL: &str = "https://crates.io/api/v1/crates?sort=recent-downloads&per_page=50";

/// Fetches top crates by recent downloads.
///
/// crates.io does not publish rate-limit headers reliably; the shared
/// fetcher applies a conservative retry policy and conditional requests.
#[derive(Debug)]
pub struct CratesConnector {
    fetcher: HttpFetcher,
    /// Crates updated within this window produce a release event.
    release_window: chrono::Duration,
}

impl CratesConnector {
    pub fn new(fetcher: HttpFetcher) -> Self {
        Self {
            fetcher,
            release_window: chrono::Duration::days(1),
        }
    }

    /// Override the release-event recency window.
    pub fn with_release_window(mut self, window: chrono::Duration) -> Self {
        self.release_window = window;
        self
    }
}

#[async_trait]
impl SourceConnector for CratesConnector {
    fn source(&self) -> Source {
        Source::Crates
    }

    async fn fetch(&self) -> Result<Vec<NormalizedOutput>, ConnectorError> {
        let body = self.fetcher.get_text(Source::Crates, TOP_URL).await?;
        let response: crate::model::CratesResponse =
            serde_json::from_str(&body).map_err(|e| ConnectorError::Parse {
                source_name: Source::Crates,
                message: format!("invalid crates.io response: {e}"),
            })?;

        let fetched_at = chrono::Utc::now();
        let out = crate::parser::parse_crates(
            response.crates,
            Source::Crates,
            fetched_at,
            self.release_window,
        );

        let mut outputs: Vec<NormalizedOutput> = out
            .releases
            .into_iter()
            .map(NormalizedOutput::Event)
            .collect();
        outputs.extend(out.crates.into_iter().map(NormalizedOutput::Crate));
        Ok(outputs)
    }
}
