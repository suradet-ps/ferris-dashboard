//! Ingestion pipeline: connector orchestration, HTTP fetching with retry,
//! normalization, idempotent persistence and scheduling.
//!
//! Every external source is isolated behind a [`SourceConnector`]; a failure
//! in one source MUST NOT abort the others (AGENTS.md §10).

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod connector;
pub mod http;
pub mod pipeline;
pub mod scheduler;

pub use connector::{ConnectorError, NormalizedOutput, SourceConnector};
pub use http::HttpFetcher;
pub use pipeline::{Pipeline, RunSummary};
pub use scheduler::Scheduler;
