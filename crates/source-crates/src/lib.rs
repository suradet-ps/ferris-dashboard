//! crates.io API connector.
//!
//! Fetches top crates by recent downloads plus a curated set of important
//! ecosystem crates, and emits release events for crates updated recently.

#![deny(unsafe_code)]

pub mod client;
pub mod error;
pub mod model;
pub mod parser;

pub use client::CratesConnector;
pub use error::CratesError;
pub use model::CrateSummary;
