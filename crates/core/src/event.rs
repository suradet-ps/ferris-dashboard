//! Event classification and the normalized event model.

use crate::source::{EventProvenance, Source};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Primary classification of an ecosystem event. Explicit categories are
/// preferred over free-form tags; see AGENTS.md §16.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// A new version of a crate, project, or the compiler.
    Release,
    /// A security advisory or vulnerability report.
    Security,
    /// Language design or specification activity.
    Language,
    /// Compiler or rustc activity.
    Compiler,
    /// A library publication or notable change.
    Library,
    /// A framework publication or notable change.
    Framework,
    /// Tooling (cargo, rustup, lints, CI, editors).
    Tooling,
    /// Project news, e.g. maintenance, funding, or roadmap.
    Project,
    /// Community news (TWIR, meetups, threads).
    Community,
    /// Conferences and events.
    Conference,
    /// Adoption by companies or large projects.
    Adoption,
    /// Performance related news.
    Performance,
    /// Funding announcements.
    Funding,
    /// Jobs and hiring.
    Job,
    /// Anything that does not fit a specific category.
    Other,
}

impl EventType {
    /// Stable database identity.
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::Release => "release",
            EventType::Security => "security",
            EventType::Language => "language",
            EventType::Compiler => "compiler",
            EventType::Library => "library",
            EventType::Framework => "framework",
            EventType::Tooling => "tooling",
            EventType::Project => "project",
            EventType::Community => "community",
            EventType::Conference => "conference",
            EventType::Adoption => "adoption",
            EventType::Performance => "performance",
            EventType::Funding => "funding",
            EventType::Job => "job",
            EventType::Other => "other",
        }
    }

    /// Human-readable label for the dashboard.
    pub fn label(&self) -> &'static str {
        match self {
            EventType::Release => "Release",
            EventType::Security => "Security",
            EventType::Language => "Language",
            EventType::Compiler => "Compiler",
            EventType::Library => "Library",
            EventType::Framework => "Framework",
            EventType::Tooling => "Tooling",
            EventType::Project => "Project",
            EventType::Community => "Community",
            EventType::Conference => "Conference",
            EventType::Adoption => "Adoption",
            EventType::Performance => "Performance",
            EventType::Funding => "Funding",
            EventType::Job => "Job",
            EventType::Other => "Other",
        }
    }
}

/// A normalized ecosystem event.
///
/// The same logical event may be reported by several sources; deduplication
/// and clustering collapse those reports into one `EcosystemEvent` with
/// multiple provenance entries (AGENTS.md §15).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemEvent {
    /// Stable database identifier.
    pub id: EcosystemEventId,
    /// Provenance of the first report that created this event.
    pub source: Source,
    /// Deterministic primary classification.
    pub event_type: EventType,
    /// Human-readable headline (untrusted external content, escaped on render).
    pub title: String,
    /// Canonical URL of the original report.
    pub url: String,
    /// Short description, sanitized, treated as untrusted input.
    pub summary: Option<String>,
    /// When the underlying item was published (source clock).
    pub published_at: Option<DateTime<Utc>>,
    /// Canonical source identifier that created this event (dedup key).
    pub source_id: String,
    /// All reports of this event, across sources.
    pub provenance: Vec<EventProvenance>,
}

impl EcosystemEvent {
    /// Timestamp used for ordering and retry bookkeeping: the fetch time of
    /// the first report, or `now` when no provenance is recorded.
    pub fn fetched_at(&self) -> DateTime<Utc> {
        self.provenance
            .first()
            .map(|p| p.fetched_at)
            .unwrap_or_else(Utc::now)
    }
}

/// Identifier of a normalized ecosystem event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EcosystemEventId(pub Uuid);

impl EcosystemEventId {
    /// Generate a fresh event id.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EcosystemEventId {
    fn default() -> Self {
        Self::new()
    }
}
