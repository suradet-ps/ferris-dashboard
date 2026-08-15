//! End-to-end ingestion checks: fixture → normalized domain output.

use ferris_core::Source;
use ferris_source_crates::CrateSummary;
use ferris_source_github::GithubRepo;
use ferris_source_rfc::parse_rfc_name;

#[test]
fn rss_blog_and_twir_normalize_to_events() {
    let blog = include_str!(
        "../../crates/source-rss/fixtures/rust-blog.xml"
    );
    let twir = include_str!("../../crates/source-rss/fixtures/twir.xml");

    let blog_events = ferris_source_rss::parse_feed(
        blog,
        Source::RustBlog,
        "https://blog.rust-lang.org/feed.xml",
    )
    .unwrap();
    let twir_events =
        ferris_source_rss::parse_feed(twir, Source::Twir, "https://this-week-in-rust.org/rss.xml")
            .unwrap();

    assert!(!blog_events.is_empty());
    assert!(!twir_events.is_empty());
    // The same logical event reported by two feeds stays two events until
    // clustering merges them (AGENTS.md §47); dedup keys differ per source.
    let blog_ids: Vec<&str> = blog_events.iter().map(|e| e.source_id.as_str()).collect();
    let twir_ids: Vec<&str> = twir_events.iter().map(|e| e.source_id.as_str()).collect();
    assert!(blog_ids.iter().all(|id| !twir_ids.contains(id)));
}

#[test]
fn github_fixture_normalizes_to_projects() {
    let fixture = include_str!("../../crates/source-github/fixtures/github-search.json");
    let response: serde_json::Value = serde_json::from_str(fixture).unwrap();
    let repos: Vec<GithubRepo> = serde_json::from_value(response["items"].clone()).unwrap();
    let fetched_at = chrono::Utc::now();
    let projects =
        ferris_source_github::parser::parse_repos(repos, fetched_at).unwrap();
    assert_eq!(projects.len(), 2);
    assert!(projects.iter().all(|p| !p.id.0.is_empty()));
}

#[test]
fn crates_fixture_produces_crates_and_fresh_releases() {
    let now = chrono::Utc::now();
    let summaries = vec![
        CrateSummary {
            id: "tokio".into(),
            name: "tokio".into(),
            description: Some("async runtime".into()),
            homepage: None,
            repository: None,
            downloads: 1_000_000,
            recent_downloads: 50_000,
            max_version: "1.45.0".into(),
            newest_version: "1.45.0".into(),
            updated_at: Some(now.to_rfc3339()),
            created_at: None,
        },
        CrateSummary {
            id: "serde".into(),
            name: "serde".into(),
            description: Some("serialization".into()),
            homepage: None,
            repository: None,
            downloads: 5_000_000,
            recent_downloads: 200_000,
            max_version: "1.0.220".into(),
            newest_version: "1.0.220".into(),
            updated_at: Some("2026-01-01T00:00:00Z".into()),
            created_at: None,
        },
    ];
    let out = ferris_source_crates::parser::parse_crates(
        summaries,
        Source::Crates,
        now,
        chrono::Duration::days(1),
    );
    assert_eq!(out.crates.len(), 2);
    // Only the crate updated within the window emits a release event.
    assert_eq!(out.releases.len(), 1);
    assert_eq!(out.releases[0].source, Source::Crates);
    assert_eq!(out.releases[0].title, "tokio 1.45.0 released");
}

#[test]
fn rustsec_fixture_normalizes_to_advisories() {
    let xml = include_str!("../../crates/source-rustsec/fixtures/rustsec.xml");
    let out = ferris_source_rustsec::parse_advisories(xml, chrono::Utc::now()).unwrap();
    assert!(!out.advisories.is_empty());
    let advisory = &out.advisories[0];
    assert!(advisory.id.starts_with("RUSTSEC-"));
    // Every advisory also produces a security event (same dedup key).
    assert!(out.events.iter().any(|e| e.source_id == advisory.id));
}

#[test]
fn rfc_names_are_parsed_deterministically() {
    let names = [
        "0001-rfc-process.md",
        "0369-no-regex.txt.md",
        "1000-rust-2027-edition.md",
    ];
    let parsed: Vec<_> = names.iter().filter_map(|n| parse_rfc_name(n)).collect();
    assert_eq!(parsed.len(), 3);
    assert_eq!(parsed[0].number, 1);
    assert_eq!(parsed[0].title, "RFC 0001: rfc process");
    assert_eq!(parsed[2].number, 1000);
    assert_eq!(parsed[2].title, "RFC 1000: rust 2027 edition");
}

#[test]
fn classification_is_stable_across_connectors() {
    // Deterministic classification must agree regardless of source.
    use ferris_core::classify;
    assert_eq!(
        classify("Announcing Rust 1.98", Some("new compiler release")),
        ferris_core::EventType::Release
    );
    assert_eq!(
        classify("RUSTSEC-2026-0042: integer overflow in hyper", Some("cve")),
        ferris_core::EventType::Security
    );
}
