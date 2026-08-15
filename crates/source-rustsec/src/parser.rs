//! RustSec RSS → advisory parsing.

use chrono::{DateTime, Utc};
use ferris_core::{
    EcosystemEvent, EcosystemEventId, EventProvenance, EventType, SecurityAdvisory, Source,
    SourceId,
};

/// Errors raised while parsing the advisory feed.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("feed-rs failed: {0}")]
    FeedRs(#[from] feed_rs::parser::ParseFeedError),
}

/// Parsed advisories plus matching security events.
pub struct ParseOutput {
    pub advisories: Vec<SecurityAdvisory>,
    pub events: Vec<EcosystemEvent>,
}

/// Parse the RustSec RSS feed.
///
/// The advisory id is derived from the canonical link
/// (`https://rustsec.org/advisories/RUSTSEC-YYYY-NNNN.html`); entries that
/// cannot be resolved to an id are skipped because they cannot be
/// deduplicated.
pub fn parse_advisories(xml: &str, fetched_at: DateTime<Utc>) -> Result<ParseOutput, ParseError> {
    let feed = feed_rs::parser::parse(xml.as_bytes())?;
    let mut advisories = Vec::new();
    let mut events = Vec::new();

    for entry in feed.entries {
        let link = entry
            .links
            .iter()
            .find(|l| l.rel.as_deref().unwrap_or("alternate") == "alternate")
            .or_else(|| entry.links.first())
            .map(|l| l.href.clone())
            .unwrap_or_default();

        // RUSTSEC-YYYY-NNNN appears in the canonical URL or the entry id.
        let id = link
            .rsplit('/')
            .next()
            .filter(|slug| slug.starts_with("RUSTSEC-"))
            .map(str::to_string)
            .or_else(|| {
                if entry.id.starts_with("RUSTSEC-") {
                    Some(entry.id)
                } else {
                    None
                }
            })
            .unwrap_or_default();
        if id.is_empty() {
            continue;
        }

        let title = entry
            .title
            .map(|t| t.content.trim().to_string())
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| id.clone());
        let summary = entry.summary.map(|s| s.content.trim().to_string());
        let published_at = entry.published.or(entry.updated);

        advisories.push(SecurityAdvisory {
            id: id.clone(),
            title: title.clone(),
            url: link.clone(),
            summary: summary.clone(),
            // The feed body is unstructured; affected crates are filled by
            // deeper enrichment later (AGENTS.md §45).
            affected_crates: Vec::new(),
            published_at,
            fetched_at,
        });

        let provenance = EventProvenance {
            source: Source::Rustsec,
            source_id: SourceId(id.clone()),
            source_url: link.clone(),
            published_at,
            fetched_at,
        };
        events.push(EcosystemEvent {
            id: EcosystemEventId::new(),
            source: Source::Rustsec,
            event_type: EventType::Security,
            title,
            url: link,
            summary,
            published_at,
            source_id: id,
            provenance: vec![provenance],
        });
    }
    Ok(ParseOutput { advisories, events })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferris_core::Source;

    #[test]
    fn parses_valid_feed() {
        let xml = include_str!("../fixtures/rustsec.xml");
        let out = parse_advisories(xml, Utc::now()).unwrap();
        assert!(!out.advisories.is_empty());
        let a = &out.advisories[0];
        assert!(a.id.starts_with("RUSTSEC-"), "id={}", a.id);
        assert_eq!(a.title, "Integer overflow in hyper");
        assert_eq!(out.events.len(), out.advisories.len());
        assert_eq!(out.events[0].event_type, EventType::Security);
        assert_eq!(out.events[0].source, Source::Rustsec);
    }

    #[test]
    fn malformed_feed_errors() {
        assert!(parse_advisories("not xml", Utc::now()).is_err());
    }

    #[test]
    fn entry_without_rustsec_id_is_skipped() {
        let xml = include_str!("../fixtures/rustsec-minimal.xml");
        let out = parse_advisories(xml, Utc::now()).unwrap();
        assert!(out.advisories.is_empty());
    }
}
