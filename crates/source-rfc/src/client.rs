//! RFC connector client: list RFC text files via the GitHub contents API.

use async_trait::async_trait;
use ferris_core::Source;
use ferris_ingest::{ConnectorError, HttpFetcher, NormalizedOutput, SourceConnector};

const CONTENTS_URL: &str = "https://api.github.com/repos/rust-lang/rfcs/contents/text";

/// Tracks new RFCs by listing the `text/` directory of `rust-lang/rfcs`.
#[derive(Debug)]
pub struct RfcConnector {
    fetcher: HttpFetcher,
    /// How many of the newest RFCs produce events per run.
    newest: usize,
}

impl RfcConnector {
    pub fn new(fetcher: HttpFetcher) -> Self {
        Self { fetcher, newest: 5 }
    }
}

#[async_trait]
impl SourceConnector for RfcConnector {
    fn source(&self) -> Source {
        Source::Rfc
    }

    async fn fetch(&self) -> Result<Vec<NormalizedOutput>, ConnectorError> {
        let body = self.fetcher.get_text(Source::Rfc, CONTENTS_URL).await?;
        let entries: Vec<crate::model::ContentsEntry> =
            serde_json::from_str(&body).map_err(|e| ConnectorError::Parse {
                source_name: Source::Rfc,
                message: format!("invalid contents response: {e}"),
            })?;

        let parsed: Vec<crate::parser::ParsedRfc> = entries
            .iter()
            .filter(|e| e.kind == "file")
            .filter_map(|e| crate::parser::parse_rfc_name(&e.name))
            .collect();

        let events =
            crate::parser::rfc_events(parsed, Source::Rfc, chrono::Utc::now(), self.newest);
        Ok(events.into_iter().map(NormalizedOutput::Event).collect())
    }
}
