//! GitHub REST API connector.
//!
//! Fetches top Rust repositories by stars. The connector is fully isolated:
//! fetch and parse are independent and fixture-tested; a GitHub outage only
//! degrades this source (AGENTS.md §10).

#![deny(unsafe_code)]

pub mod client;
pub mod error;
pub mod model;
pub mod parser;

pub use client::GithubConnector;
pub use error::GithubError;
pub use model::GithubRepo;
