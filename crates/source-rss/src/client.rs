//! RSS fetching: a thin wrapper over the shared retrying [`HttpFetcher`].

use async_trait::async_trait;
use ferris_core::Source;
use ferris_ingest::{ConnectorError, HttpFetcher, NormalizedOutput, SourceConnector};
use std::fmt;

/// Fetches and normalizes one RSS/Atom feed into ecosystem events.
///
/// `Rust Blog` and `This Week in Rust` are both RSS feeds; they only differ
/// in identity and URL, so one connector covers both.
#[derive(Debug)]
pub struct RssConnector {
    source: Source,
    feed_url: String,
    fetcher: HttpFetcher,
}

impl RssConnector {
    /// Connector for the official Rust blog feed.
    pub fn rust_blog(fetcher: HttpFetcher) -> Self {
        Self {
            source: Source::RustBlog,
            feed_url: "https://blog.rust-lang.org/feed.xml".into(),
            fetcher,
        }
    }

    /// Connector for This Week in Rust.
    pub fn twir(fetcher: HttpFetcher) -> Self {
        Self {
            source: Source::Twir,
            feed_url: "https://this-week-in-rust.org/rss.xml".into(),
            fetcher,
        }
    }

    /// The configured feed URL (used by the parser for fallback ids).
    pub fn feed_url(&self) -> &str {
        &self.feed_url
    }
}

#[async_trait]
impl SourceConnector for RssConnector {
    fn source(&self) -> Source {
        self.source
    }

    async fn fetch(&self) -> Result<Vec<NormalizedOutput>, ConnectorError> {
        let xml = self.fetcher.get_text(self.source, &self.feed_url).await?;
        let entries =
            crate::parser::parse_feed(&xml, self.source, &self.feed_url).map_err(|e| {
                ConnectorError::Parse {
                    source_name: self.source,
                    message: e.to_string(),
                }
            })?;
        Ok(entries.into_iter().map(NormalizedOutput::Event).collect())
    }
}

impl fmt::Display for RssConnector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RssConnector({}, {})", self.source, self.feed_url)
    }
}
