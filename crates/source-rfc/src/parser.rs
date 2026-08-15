//! RFC filename → language event conversion.

use chrono::{DateTime, Utc};
use ferris_core::{EcosystemEvent, EcosystemEventId, EventProvenance, EventType, Source, SourceId};

/// An RFC parsed from its filename.
#[derive(Debug, Clone)]
pub struct ParsedRfc {
    pub number: u64,
    pub slug: String,
    pub title: String,
    pub url: String,
}

/// Extract `NNNN-slug` information from a file name like `0001-my-rfc.md`.
pub fn parse_rfc_name(file_name: &str) -> Option<ParsedRfc> {
    let stem = file_name.strip_suffix(".md")?;
    let (number_part, slug) = stem.split_once('-')?;
    let number: u64 = number_part.parse().ok()?;
    let slug = slug.replace('-', " ");
    Some(ParsedRfc {
        number,
        slug: slug.clone(),
        title: format!("RFC {number:04}: {slug}"),
        url: format!("https://github.com/rust-lang/rfcs/blob/master/text/{stem}.md"),
    })
}

/// Convert parsed RFCs into language events, newest RFC first.
///
/// Only the `newest` most recent RFCs produce events; older RFCs are
/// skipped to avoid re-publishing history on every fetch (AGENTS.md §41
/// idempotency: the event source_id is stable per RFC).
pub fn rfc_events(
    parsed: Vec<ParsedRfc>,
    source: Source,
    fetched_at: DateTime<Utc>,
    newest: usize,
) -> Vec<EcosystemEvent> {
    let mut sorted = parsed;
    sorted.sort_by_key(|r| std::cmp::Reverse(r.number));
    let mut events = Vec::new();
    for rfc in sorted.into_iter().take(newest) {
        let source_id = format!("rfc-{:04}", rfc.number);
        let provenance = EventProvenance {
            source,
            source_id: SourceId(source_id.clone()),
            source_url: rfc.url.clone(),
            published_at: None,
            fetched_at,
        };
        events.push(EcosystemEvent {
            id: EcosystemEventId::new(),
            source,
            event_type: EventType::Language,
            title: rfc.title,
            url: rfc.url,
            summary: Some(rfc.slug),
            published_at: None,
            source_id,
            provenance: vec![provenance],
        });
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn parses_standard_rfc_name() {
        let rfc = parse_rfc_name("0001-my-awesome-rfc.md").unwrap();
        assert_eq!(rfc.number, 1);
        assert_eq!(rfc.title, "RFC 0001: my awesome rfc");
        assert_eq!(
            rfc.url,
            "https://github.com/rust-lang/rfcs/blob/master/text/0001-my-awesome-rfc.md"
        );
    }

    #[test]
    fn rejects_non_rfc_names() {
        assert!(parse_rfc_name("README.md").is_none());
        assert!(parse_rfc_name("not-a-number.md").is_none());
        assert!(parse_rfc_name("0001.txt").is_none());
    }

    #[test]
    fn newest_n_rfcs_emit_events() {
        let fetched = Utc.with_ymd_and_hms(2026, 8, 15, 12, 0, 0).unwrap();
        let parsed = vec![
            parse_rfc_name("0001-first.md").unwrap(),
            parse_rfc_name("0002-second.md").unwrap(),
            parse_rfc_name("0003-third.md").unwrap(),
        ];
        let events = rfc_events(parsed, Source::Rfc, fetched, 2);
        assert_eq!(events.len(), 2);
        assert!(events[0].title.starts_with("RFC 0003"));
        assert_eq!(events[0].event_type, EventType::Language);
        assert_eq!(events[0].source_id, "rfc-0003");
    }
}
