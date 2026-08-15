//! RSS/Atom source connector (Rust Blog, This Week in Rust).
//!
//! Both feeds share the same parser; only the identity differs.

#![deny(unsafe_code)]

pub mod client;
pub mod parser;

pub use client::RssConnector;
pub use parser::parse_feed;
