//! Source freshness and health reporting.

use crate::source::Source;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Health of a single source as observed by the ingestion worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceStatus {
    /// Last fetch succeeded.
    Healthy,
    /// Last fetch failed; worker will retry with backoff.
    Degraded,
    /// The source is disabled in configuration.
    Disabled,
}

impl SourceStatus {
    /// Human-readable label for the dashboard.
    pub fn label(&self) -> &'static str {
        match self {
            SourceStatus::Healthy => "healthy",
            SourceStatus::Degraded => "degraded",
            SourceStatus::Disabled => "disabled",
        }
    }
}

/// Freshness + health of one source, exposed by the API for the dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceHealth {
    /// The source this health record describes.
    pub source: Source,
    /// Current status derived from the last fetch.
    pub status: SourceStatus,
    /// When the last fetch succeeded (data age anchor).
    pub last_fetched_at: Option<DateTime<Utc>>,
    /// When the last fetch was attempted, success or failure.
    pub last_attempted_at: Option<DateTime<Utc>>,
    /// Human-readable error of the last failed fetch.
    pub last_error: Option<String>,
    /// Duration of the last fetch.
    pub last_duration_ms: Option<u64>,
    /// Recommended refresh interval in minutes.
    pub interval_minutes: u64,
}
