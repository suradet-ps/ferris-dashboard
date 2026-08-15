//! GitHub connector client: fetch trending Rust repositories.

use async_trait::async_trait;
use ferris_core::Source;
use ferris_ingest::{ConnectorError, HttpFetcher, NormalizedOutput, SourceConnector};

const SEARCH_URL: &str =
    "https://api.github.com/search/repositories?q=language:rust&sort=stars&order=desc&per_page=50";

/// Fetches top Rust repositories from the GitHub search API.
///
/// Rate limits: unauthenticated requests are capped at 60 req/h; providing
/// a `GITHUB_TOKEN` raises that to 5000 req/h. Retry, backoff and
/// rate-limit detection come from the shared [`HttpFetcher`].
#[derive(Debug)]
pub struct GithubConnector {
    fetcher: HttpFetcher,
}

impl GithubConnector {
    pub fn new(fetcher: HttpFetcher) -> Self {
        Self { fetcher }
    }
}

#[async_trait]
impl SourceConnector for GithubConnector {
    fn source(&self) -> Source {
        Source::Github
    }

    async fn fetch(&self) -> Result<Vec<NormalizedOutput>, ConnectorError> {
        let body = self.fetcher.get_text(Source::Github, SEARCH_URL).await?;
        let response: crate::model::SearchResponse =
            serde_json::from_str(&body).map_err(|e| ConnectorError::Parse {
                source_name: Source::Github,
                message: format!("invalid search response: {e}"),
            })?;

        let projects = crate::parser::parse_repos(response.items, chrono::Utc::now())?;
        Ok(projects
            .into_iter()
            .map(NormalizedOutput::Project)
            .collect())
    }
}
