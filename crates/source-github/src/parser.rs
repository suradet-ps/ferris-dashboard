//! GitHub → normalized domain conversion.

use crate::model::GithubRepo;
use ferris_core::{Project, ProjectId};
use ferris_ingest::ConnectorError;

/// Convert raw GitHub search results into normalized project snapshots.
///
/// Repos without a push date are skipped (they carry no activity signal).
pub fn parse_repos(
    repos: Vec<GithubRepo>,
    fetched_at: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<Project>, ConnectorError> {
    let mut projects = Vec::with_capacity(repos.len());
    for repo in repos {
        let pushed_at = repo
            .pushed_at
            .as_deref()
            .map(|s| {
                chrono::DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&chrono::Utc))
            })
            .transpose()
            .map_err(|e| ConnectorError::Parse {
                source_name: ferris_core::Source::Github,
                message: format!("bad pushed_at for {}: {e}", repo.full_name),
            })?;

        projects.push(Project {
            id: ProjectId(repo.full_name.clone()),
            name: repo.full_name,
            description: repo.description,
            html_url: repo.html_url,
            stars: repo.stargazers_count,
            forks: repo.forks_count,
            open_issues: repo.open_issues_count,
            language: repo.language,
            archived: repo.archived,
            pushed_at,
            fetched_at,
        });
    }
    Ok(projects)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn parses_repos_from_fixture() {
        let fixture = include_str!("../fixtures/github-search.json");
        let response: crate::model::SearchResponse = serde_json::from_str(fixture).unwrap();
        let fetched_at = Utc.with_ymd_and_hms(2026, 8, 15, 0, 0, 0).unwrap();

        let projects = parse_repos(response.items, fetched_at).unwrap();
        assert_eq!(projects.len(), 2);
        assert_eq!(projects[0].id.0, "tokio-rs/tokio");
        assert_eq!(projects[0].stars, 30_000);
        assert!(projects[0].pushed_at.is_some());
        assert_eq!(projects[1].id.0, "rust-lang/rust");
        assert!(!projects[1].archived);
    }

    #[test]
    fn bad_pushed_at_is_rejected() {
        let repo = GithubRepo {
            id: 1,
            full_name: "a/b".into(),
            name: "b".into(),
            description: None,
            html_url: "https://github.com/a/b".into(),
            stargazers_count: 1,
            forks_count: 0,
            open_issues_count: 0,
            language: Some("Rust".into()),
            archived: false,
            pushed_at: Some("not-a-date".into()),
        };
        let result = parse_repos(vec![repo], Utc::now());
        assert!(result.is_err());
    }
}
