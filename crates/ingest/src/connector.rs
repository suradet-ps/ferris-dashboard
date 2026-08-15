//! The common connector abstraction (AGENTS.md §7).
//!
//! A connector MUST NOT leak source-specific types into the domain: it
//! fetches raw data, parses it, and produces only [`NormalizedOutput`].

use async_trait::async_trait;
use ferris_core::{EcosystemEvent, Project, SecurityAdvisory, Source};
use std::fmt;
use std::time::Duration;

/// A crate snapshot (kept local to avoid a core type cycle in this module).
pub use ferris_core::Crate as CrateSnapshot;

/// Failure modes shared by every connector. Connectors map their own
/// errors onto these so the pipeline can apply uniform retry policy.
#[derive(Debug, thiserror::Error)]
pub enum ConnectorError {
    /// Transport-level failure (timeout, connection refused, DNS).
    #[error("http error while fetching {source_name}: {message}")]
    Http {
        /// The source that failed.
        source_name: Source,
        /// Human-readable transport error.
        message: String,
    },
    /// The source asked us to slow down (429/403). `retry_after` is
    /// respected when the source provides it.
    #[error("rate limited by {source_name}")]
    RateLimited {
        /// The source that was rate limited.
        source_name: Source,
        /// Suggested retry delay.
        retry_after: Option<Duration>,
    },
    /// Payload could not be parsed or failed validation.
    #[error("failed to parse data from {source_name}: {message}")]
    Parse {
        /// The source with malformed data.
        source_name: Source,
        /// Parser error detail.
        message: String,
    },
    /// The source is disabled by configuration.
    #[error("source {0} is disabled")]
    Disabled(Source),
    /// Anything else.
    #[error("connector error: {0}")]
    Other(String),
}

/// Normalized output of a connector. This is the boundary between the
/// source-specific world and the domain world.
#[derive(Debug)]
pub enum NormalizedOutput {
    /// An ecosystem event (Rust Blog, TWIR, releases, ...).
    Event(EcosystemEvent),
    /// A GitHub project snapshot.
    Project(Project),
    /// A crates.io crate snapshot.
    Crate(CrateSnapshot),
    /// A security advisory (RustSec).
    Advisory(SecurityAdvisory),
}

/// Every source MUST implement this abstraction. Implementations must be
/// `Send + Sync` so the pipeline can run sources in parallel.
#[async_trait]
pub trait SourceConnector: Send + Sync + fmt::Debug {
    /// The identity of the source this connector represents.
    fn source(&self) -> Source;

    /// Fetch and normalize items from the external source.
    ///
    /// The connector is responsible for fetching, parsing, validating and
    /// normalizing. It returns already-normalized domain values; parse
    /// problems MUST be reported as [`ConnectorError::Parse`], not panics.
    async fn fetch(&self) -> Result<Vec<NormalizedOutput>, ConnectorError>;
}
