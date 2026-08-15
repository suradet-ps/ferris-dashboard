//! RustSec connector client.

use async_trait::async_trait;
use ferris_core::Source;
use ferris_ingest::{ConnectorError, HttpFetcher, NormalizedOutput, SourceConnector};

const FEED_URL: &str = "https://rustsec.org/advisories/rss.xml";

/// Fetches and normalizes the RustSec advisory feed.
#[derive(Debug)]
pub struct RustsecConnector {
    fetcher: HttpFetcher,
}

impl RustsecConnector {
    pub fn new(fetcher: HttpFetcher) -> Self {
        Self { fetcher }
    }
}

#[async_trait]
impl SourceConnector for RustsecConnector {
    fn source(&self) -> Source {
        Source::Rustsec
    }

    async fn fetch(&self) -> Result<Vec<NormalizedOutput>, ConnectorError> {
        let xml = self.fetcher.get_text(Source::Rustsec, FEED_URL).await?;
        let out = crate::parser::parse_advisories(&xml, chrono::Utc::now()).map_err(|e| {
            ConnectorError::Parse {
                source_name: Source::Rustsec,
                message: e.to_string(),
            }
        })?;

        let mut outputs: Vec<NormalizedOutput> = out
            .events
            .into_iter()
            .map(NormalizedOutput::Event)
            .collect();
        outputs.extend(out.advisories.into_iter().map(NormalizedOutput::Advisory));
        Ok(outputs)
    }
}
