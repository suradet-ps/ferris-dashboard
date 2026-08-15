//! RSS/Atom → normalized event parsing.
//!
//! Titles, summaries and URLs are treated as untrusted input (AGENTS.md
//! §32): text is kept as plain text, URLs are validated.

use chrono::Utc;
use ferris_core::{EcosystemEvent, EcosystemEventId, EventProvenance, Source, SourceId, classify};
use url::Url;

/// Errors raised while parsing a feed.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("feed-rs failed: {0}")]
    FeedRs(#[from] feed_rs::parser::ParseFeedError),
}

/// Parse a feed document into normalized ecosystem events.
///
/// Entries without a usable id or link are skipped: they cannot be
/// deduplicated reliably (AGENTS.md §15).
pub fn parse_feed(
    xml: &str,
    source: Source,
    feed_url: &str,
) -> Result<Vec<EcosystemEvent>, ParseError> {
    let feed = feed_rs::parser::parse(xml.as_bytes())?;
    let fetched_at = Utc::now();

    let mut events = Vec::new();
    for entry in feed.entries {
        let link = entry
            .links
            .iter()
            .find(|l| l.rel.as_deref().unwrap_or("alternate") == "alternate")
            .or_else(|| entry.links.first())
            .and_then(|l| validate_url(&l.href))
            .or_else(|| validate_url(feed_url))
            .unwrap_or_default();

        let entry_id = if !entry.id.is_empty() {
            entry.id
        } else if !link.is_empty() {
            link.clone()
        } else {
            continue;
        };

        let title = entry
            .title
            .map(|t| t.content.trim().to_string())
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| "Untitled".into());

        let summary = entry.summary.map(|s| s.content.trim().to_string());
        let published_at = entry.published.or(entry.updated);

        let provenance = EventProvenance {
            source,
            source_id: SourceId(entry_id.clone()),
            source_url: link.clone(),
            published_at,
            fetched_at,
        };

        let event_type = classify(&title, summary.as_deref());

        events.push(EcosystemEvent {
            id: EcosystemEventId::new(),
            source,
            event_type,
            title,
            url: link,
            summary,
            published_at,
            source_id: entry_id,
            provenance: vec![provenance],
        });
    }
    Ok(events)
}

/// Validate a URL string. Returns `Some` when it parses and is http(s).
fn validate_url(raw: &str) -> Option<String> {
    let url = Url::parse(raw).ok()?;
    match url.scheme() {
        "http" | "https" => Some(url.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferris_core::{EventType, Source};

    #[test]
    fn parses_valid_rss_feed() {
        let xml = include_str!("../fixtures/rust-blog.xml");
        let events =
            parse_feed(xml, Source::RustBlog, "https://blog.rust-lang.org/feed.xml").unwrap();
        assert!(!events.is_empty(), "expected entries");
        let first = &events[0];
        assert_eq!(first.source, Source::RustBlog);
        assert!(!first.title.is_empty());
        assert!(first.url.starts_with("https://"));
        assert!(!first.source_id.is_empty());
        assert_eq!(first.provenance.len(), 1);
    }

    #[test]
    fn twir_feed_is_classified_as_community() {
        let xml = include_str!("../fixtures/twir.xml");
        let events =
            parse_feed(xml, Source::Twir, "https://this-week-in-rust.org/rss.xml").unwrap();
        assert!(!events.is_empty());
        assert_eq!(events[0].event_type, EventType::Community);
    }

    #[test]
    fn malformed_feed_returns_error() {
        let xml = include_str!("../fixtures/malformed.xml");
        assert!(parse_feed(xml, Source::Twir, "https://x.test/feed").is_err());
    }

    #[test]
    fn missing_fields_are_tolerated() {
        let xml = include_str!("../fixtures/minimal.xml");
        let events =
            parse_feed(xml, Source::RustBlog, "https://blog.rust-lang.org/feed.xml").unwrap();
        // One entry without title/summary must still normalize.
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].title, "Untitled");
    }
}
