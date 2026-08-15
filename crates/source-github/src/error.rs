//! Connector-specific errors, mapped onto [`ConnectorError`] at the
//! connector boundary.

use ferris_ingest::ConnectorError;

/// Errors raised by the GitHub connector.
#[derive(Debug, thiserror::Error)]
pub enum GithubError {
    #[error("unexpected JSON structure: {0}")]
    Malformed(#[from] serde_json::Error),
    #[error("repository has no push date: {0}")]
    MissingPushedAt(String),
}

impl From<GithubError> for ConnectorError {
    fn from(err: GithubError) -> Self {
        ConnectorError::Parse {
            source_name: ferris_core::Source::Github,
            message: err.to_string(),
        }
    }
}
