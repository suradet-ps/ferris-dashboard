//! Source identities and provenance metadata.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A normalized source identifier. One source may produce multiple event
/// types (e.g. GitHub produces releases and repository metrics).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// Official Rust blog: https://blog.rust-lang.org/feed.xml
    RustBlog,
    /// This Week in Rust: https://this-week-in-rust.org/rss.xml
    Twir,
    /// crates.io API: https://crates.io/api/v1/
    Crates,
    /// GitHub REST API: https://api.github.com/
    Github,
    /// RustSec advisories: https://rustsec.org/advisories/rss.xml
    Rustsec,
    /// Rust RFC repository: https://github.com/rust-lang/rfcs
    Rfc,
    /// Defensive fallback for unknown rows; never scheduled, never fetched.
    Other,
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Source::RustBlog => "rust_blog",
            Source::Twir => "twir",
            Source::Crates => "crates",
            Source::Github => "github",
            Source::Rustsec => "rustsec",
            Source::Rfc => "rfc",
            Source::Other => "other",
        })
    }
}

impl Source {
    /// All known sources, used for health reporting and scheduling.
    pub const ALL: [Source; 6] = [
        Source::RustBlog,
        Source::Twir,
        Source::Crates,
        Source::Github,
        Source::Rustsec,
        Source::Rfc,
    ];

    /// Stable database identity (lowercase ascii, unique).
    pub fn as_str(&self) -> &'static str {
        match self {
            Source::RustBlog => "rust_blog",
            Source::Twir => "twir",
            Source::Crates => "crates",
            Source::Github => "github",
            Source::Rustsec => "rustsec",
            Source::Rfc => "rfc",
            Source::Other => "other",
        }
    }

    /// Human-readable label for the dashboard.
    pub fn label(&self) -> &'static str {
        match self {
            Source::RustBlog => "Rust Blog",
            Source::Twir => "This Week in Rust",
            Source::Crates => "crates.io",
            Source::Github => "GitHub",
            Source::Rustsec => "RustSec",
            Source::Rfc => "Rust RFCs",
            Source::Other => "Unknown",
        }
    }

    /// Whether this source is schedulable by the ingestion worker.
    pub fn is_schedulable(&self) -> bool {
        !matches!(self, Source::Other)
    }

    /// Default refresh interval in minutes (AGENTS.md §12).
    ///
    /// GitHub changes fast; RFCs and advisories change slowly. The worker
    /// can override any of these via environment configuration.
    pub fn default_interval_minutes(&self) -> u64 {
        match self {
            Source::Github => 10,
            Source::Crates => 15,
            Source::RustBlog => 30,
            Source::Twir => 30,
            Source::Rustsec => 360,
            Source::Rfc => 360,
            Source::Other => u64::MAX, // never scheduled
        }
    }
}

/// The unique identity of one normalized event within its source.
///
/// This is the primary deduplication key: the same logical event arriving
/// twice from the same source must carry the same `SourceId`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceId(pub String);

impl fmt::Display for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Provenance of a normalized event: where it came from and when.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventProvenance {
    /// Source that produced this event.
    pub source: Source,
    /// Source-specific identifier (dedup key within the source).
    pub source_id: SourceId,
    /// Canonical URL of the original item.
    pub source_url: String,
    /// When the source item was published.
    pub published_at: Option<DateTime<Utc>>,
    /// When the item was fetched from the source.
    pub fetched_at: DateTime<Utc>,
}
