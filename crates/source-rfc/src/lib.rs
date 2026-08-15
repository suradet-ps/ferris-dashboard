//! Rust RFC tracking connector.
//!
//! Lists RFC text files from `rust-lang/rfcs` via the GitHub contents API
//! and normalizes recent RFCs (by RFC number) into language events. The RFC
//! number ordering is the deterministic, cheap signal: RFC numbers increase
//! over time, so the newest files are the newest RFCs.

#![deny(unsafe_code)]

pub mod client;
pub mod model;
pub mod parser;

pub use client::RfcConnector;
pub use model::ContentsEntry;
