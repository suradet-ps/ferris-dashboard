//! RustSec advisory connector.
//!
//! Consumes the RustSec advisory RSS feed and normalizes entries into
//! security advisories plus security ecosystem events.

#![deny(unsafe_code)]

pub mod client;
pub mod parser;

pub use client::RustsecConnector;
pub use parser::parse_advisories;
