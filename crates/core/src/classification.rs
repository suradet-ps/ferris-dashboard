//! Deterministic event classification.
//!
//! Classification is keyword-driven and fully deterministic. It is the
//! foundation layer; LLM enrichment (if ever added) must layer on top and
//! never be required for basic classification (AGENTS.md §45).

use crate::event::EventType;

/// Classify a title + summary into a primary [`EventType`].
///
/// Order matters: the first matching group wins. Groups are ordered from
/// most specific to least specific.
pub fn classify(title: &str, summary: Option<&str>) -> EventType {
    let haystack = format!("{} {}", title, summary.unwrap_or_default()).to_lowercase();

    if contains_any(
        &haystack,
        &[
            "cve",
            "advisory",
            "advisories",
            "vulnerability",
            "vulnerabilities",
            "security fix",
            "rustsec",
        ],
    ) {
        EventType::Security
    } else if contains_any(&haystack, &["job", "hiring", "we are hiring", "careers"]) {
        EventType::Job
    } else if contains_any(
        &haystack,
        &[
            "conference",
            "rustconf",
            "meetup",
            "summit",
            "call for talks",
            "cfs",
            "cfp",
        ],
    ) {
        EventType::Conference
    } else if contains_any(
        &haystack,
        &["funding", "grant", "sponsor", "sponsorship", "foundation"],
    ) {
        EventType::Funding
    } else if contains_any(
        &haystack,
        &[
            "adopted",
            "adoption",
            "in production",
            "migrating to rust",
            "rewritten in rust",
        ],
    ) {
        EventType::Adoption
    } else if contains_any(
        &haystack,
        &[
            "rustc",
            "compiler",
            "llvm",
            "miri",
            "const evaluation",
            "codegen",
        ],
    ) {
        EventType::Compiler
    } else if contains_any(
        &haystack,
        &[
            "performance",
            "faster",
            "speedup",
            "benchmark",
            "optimization",
            "optimising",
            "optimizing",
        ],
    ) {
        EventType::Performance
    } else if contains_any(
        &haystack,
        &[
            "edition",
            "rfc",
            "trait",
            "type system",
            "borrow checker",
            "language changes",
            "language design",
            "lifetime",
        ],
    ) {
        EventType::Language
    } else if contains_any(
        &haystack,
        &[
            "cargo",
            "rustup",
            "clippy",
            "rustfmt",
            "lsp",
            "rust-analyzer",
            "editors",
            "intellij",
            "vscode",
            "toolchain",
            "lint",
        ],
    ) {
        EventType::Tooling
    } else if contains_any(
        &haystack,
        &[
            "framework",
            "actix",
            "axum",
            "tokio",
            "rocket",
            "bevy",
            "egui",
            "iced",
            "leptos",
            "dioxus",
            "tauri",
            "warp",
            "poem",
            "salvo",
        ],
    ) {
        EventType::Framework
    } else if contains_any(
        &haystack,
        &[
            "library",
            "crate",
            "crates.io",
            "new version",
            "released",
            "release",
            "v0.",
            "version",
        ],
    ) {
        EventType::Release
    } else if contains_any(
        &haystack,
        &[
            "this week in rust",
            "community",
            "announcement",
            "blog post",
            "survey",
            "thread",
            "roundup",
            "newsletter",
        ],
    ) {
        EventType::Community
    } else if contains_any(
        &haystack,
        &[
            "maintenance",
            "roadmap",
            "deprecated",
            "deprecation",
            "archived",
            "maintainer",
            "stewardship",
        ],
    ) {
        EventType::Project
    } else {
        EventType::Other
    }
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| haystack.contains(n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_release() {
        assert_eq!(
            classify("Announcing Rust 1.98", Some("the new release is out")),
            EventType::Release
        );
    }

    #[test]
    fn security_beats_release() {
        assert_eq!(
            classify(
                "CVE-2026-0001 in hyper",
                Some("vulnerability in a widely used crate")
            ),
            EventType::Security
        );
    }

    #[test]
    fn classifies_compiler() {
        assert_eq!(
            classify("rustc 1.98 performance improvements", None),
            EventType::Compiler
        );
    }

    #[test]
    fn classifies_community() {
        assert_eq!(
            classify("This Week in Rust 598", None),
            EventType::Community
        );
    }

    #[test]
    fn falls_back_to_other() {
        assert_eq!(classify("Random post about llamas", None), EventType::Other);
    }

    #[test]
    fn framework_beats_release() {
        assert_eq!(
            classify("axum 0.9 released", Some("new framework version")),
            EventType::Framework
        );
    }
}
