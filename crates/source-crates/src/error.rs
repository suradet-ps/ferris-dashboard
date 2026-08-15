//! Connector-specific errors.

use ferris_ingest::ConnectorError;

/// Errors raised by the crates.io connector.
#[derive(Debug, thiserror::Error)]
pub enum CratesError {
    #[error("unexpected JSON structure: {0}")]
    Malformed(#[from] serde_json::Error),
}

impl From<CratesError> for ConnectorError {
    fn from(err: CratesError) -> Self {
        ConnectorError::Parse {
            source_name: ferris_core::Source::Crates,
            message: err.to_string(),
        }
    }
}
