//! GitHub API response models (source-specific, never leak into the domain).

use serde::Deserialize;

/// Minimal projection of the GitHub search API response.
#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub total_count: u64,
    pub items: Vec<GithubRepo>,
}

/// One repository from the search API.
#[derive(Debug, Deserialize)]
pub struct GithubRepo {
    pub id: u64,
    pub full_name: String,
    pub name: String,
    pub description: Option<String>,
    pub html_url: String,
    pub stargazers_count: u64,
    pub forks_count: u64,
    pub open_issues_count: u64,
    pub language: Option<String>,
    pub archived: bool,
    pub pushed_at: Option<String>,
}
