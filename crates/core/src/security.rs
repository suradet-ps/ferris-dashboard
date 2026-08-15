//! Security advisory model (RustSec).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A normalized RustSec advisory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAdvisory {
    /// Unique advisory id (e.g. `RUSTSEC-2026-0001`).
    pub id: String,
    /// Advisory headline (untrusted external content).
    pub title: String,
    /// Canonical advisory URL.
    pub url: String,
    /// The advisory body, sanitized, treated as untrusted input.
    pub summary: Option<String>,
    /// Crates affected by this advisory.
    pub affected_crates: Vec<String>,
    /// When the advisory was published.
    pub published_at: Option<DateTime<Utc>>,
    /// When the advisory was fetched.
    pub fetched_at: DateTime<Utc>,
}
