//! Project, crate, and momentum models.

use crate::id::{CrateId, ProjectId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A project observed by GitHub: stars, forks, contributors, commit activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Canonical `owner/name` identifier.
    pub id: ProjectId,
    /// `owner/name` display name.
    pub name: String,
    /// Repository description (untrusted external content).
    pub description: Option<String>,
    /// Canonical GitHub URL.
    pub html_url: String,
    /// Star count at snapshot time.
    pub stars: u64,
    /// Fork count at snapshot time.
    pub forks: u64,
    /// Open issue count at snapshot time.
    pub open_issues: u64,
    /// Primary language reported by GitHub.
    pub language: Option<String>,
    /// Whether the repository is archived (excluded from rankings).
    pub archived: bool,
    /// Last push time reported by GitHub.
    pub pushed_at: Option<DateTime<Utc>>,
    /// When this snapshot was taken.
    pub fetched_at: DateTime<Utc>,
}

/// A crate observed on crates.io.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Crate {
    /// crates.io crate name (id).
    pub id: CrateId,
    /// Display name.
    pub name: String,
    /// Crate description (untrusted external content).
    pub description: Option<String>,
    /// Homepage URL when provided.
    pub homepage: Option<String>,
    /// Source repository URL when provided.
    pub repository: Option<String>,
    /// Total lifetime downloads.
    pub downloads: u64,
    /// Downloads in the last 90 days (velocity proxy).
    pub recent_downloads: u64,
    /// Highest published version.
    pub max_version: String,
    /// When crates.io last updated this crate.
    pub updated_at: Option<DateTime<Utc>>,
    /// When this snapshot was taken.
    pub fetched_at: DateTime<Utc>,
}

/// A single historical observation of a metric (stars, downloads, ...).
///
/// Time-series observations enable velocity, acceleration and trend
/// detection (AGENTS.md §19).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    /// When the observation was recorded.
    pub timestamp: DateTime<Utc>,
    /// The observed value.
    pub value: f64,
}

/// Explainable momentum score. Every component is visible so the dashboard
/// can answer "why is this project trending?" (AGENTS.md §18).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    /// Star-velocity component (0..=1).
    pub stars: f64,
    /// Download-velocity component (0..=1).
    pub downloads: f64,
    /// Contributor-growth component (0..=1).
    pub contributors: f64,
    /// Release-activity component (0..=1).
    pub releases: f64,
    /// Community-signal component (0..=1).
    pub community: f64,
    /// Weighted total (0..=1).
    pub total: f64,
}

impl ScoreBreakdown {
    /// The maximum reachable component weight (scoring crate defines the
    /// exact formula; this is only the normalized 0..1 view used by the UI).
    pub const MAX_COMPONENT: f64 = 1.0;

    /// A score with no observations is zero and explainable as "no data".
    pub fn zero() -> Self {
        Self {
            stars: 0.0,
            downloads: 0.0,
            contributors: 0.0,
            releases: 0.0,
            community: 0.0,
            total: 0.0,
        }
    }
}
