//! crates.io → normalized domain conversion.

use crate::model::CrateSummary;
use chrono::{DateTime, Utc};
use ferris_core::{
    Crate, CrateId, EcosystemEvent, EcosystemEventId, EventProvenance, Source, SourceId, classify,
};

/// Convert raw crate summaries into normalized crate snapshots and,
/// for crates updated within `release_window`, release events.
pub struct ParseOutput {
    pub crates: Vec<Crate>,
    /// Release events for crates updated within the recency window.
    pub releases: Vec<EcosystemEvent>,
}

/// Parse crate summaries into domain values.
///
/// `release_window` (e.g. 24h) determines which crates produce a
/// "release" ecosystem event, so releases are only surfaced when fresh.
pub fn parse_crates(
    summaries: Vec<CrateSummary>,
    source: Source,
    fetched_at: DateTime<Utc>,
    release_window: chrono::Duration,
) -> ParseOutput {
    let mut crates = Vec::with_capacity(summaries.len());
    let mut releases = Vec::new();

    for summary in summaries {
        let updated_at = summary
            .updated_at
            .as_deref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        crates.push(Crate {
            id: CrateId(summary.id.clone()),
            name: summary.name.clone(),
            description: summary.description.clone(),
            homepage: summary.homepage.clone(),
            repository: summary.repository.clone(),
            downloads: summary.downloads,
            recent_downloads: summary.recent_downloads,
            max_version: summary.max_version.clone(),
            updated_at,
            fetched_at,
        });

        let recently_updated = updated_at
            .map(|t| fetched_at.signed_duration_since(t) <= release_window)
            .unwrap_or(false);
        if recently_updated {
            let title = format!("{} {} released", summary.name, summary.newest_version);
            let url = format!(
                "https://crates.io/crates/{}/{}",
                summary.name, summary.newest_version
            );
            let source_id = format!("{}-{}", summary.name, summary.newest_version);
            let provenance = EventProvenance {
                source,
                source_id: SourceId(source_id.clone()),
                source_url: url.clone(),
                published_at: updated_at,
                fetched_at,
            };
            releases.push(EcosystemEvent {
                id: EcosystemEventId::new(),
                source,
                event_type: classify(&title, summary.description.as_deref()),
                title,
                url,
                summary: summary.description,
                published_at: updated_at,
                source_id,
                provenance: vec![provenance],
            });
        }
    }

    ParseOutput { crates, releases }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(name: &str, updated_at: Option<&str>) -> CrateSummary {
        CrateSummary {
            id: name.into(),
            name: name.into(),
            description: Some("A crate".into()),
            homepage: None,
            repository: None,
            downloads: 1_000_000,
            recent_downloads: 50_000,
            max_version: "1.2.3".into(),
            newest_version: "1.2.3".into(),
            updated_at: updated_at.map(str::to_string),
            created_at: None,
        }
    }

    #[test]
    fn fresh_crate_emits_release_event() {
        let fetched = Utc::now();
        let summaries = vec![summary("tokio", Some("2026-08-15T00:00:00Z"))];
        let out = parse_crates(
            summaries,
            Source::Crates,
            fetched,
            chrono::Duration::days(1),
        );
        assert_eq!(out.crates.len(), 1);
        assert_eq!(out.releases.len(), 1);
        assert_eq!(out.releases[0].title, "tokio 1.2.3 released");
        assert!(out.releases[0].url.starts_with("https://crates.io/crates/"));
        assert!(!out.releases[0].source_id.is_empty());
    }

    #[test]
    fn stale_crate_emits_no_release_event() {
        let fetched = Utc::now();
        let summaries = vec![summary("serde", Some("2026-01-01T00:00:00Z"))];
        let out = parse_crates(
            summaries,
            Source::Crates,
            fetched,
            chrono::Duration::days(1),
        );
        assert_eq!(out.releases.len(), 0);
    }

    #[test]
    fn missing_update_date_never_emits_release() {
        let fetched = Utc::now();
        let summaries = vec![summary("rand", None)];
        let out = parse_crates(
            summaries,
            Source::Crates,
            fetched,
            chrono::Duration::days(1),
        );
        assert_eq!(out.releases.len(), 0);
    }
}
