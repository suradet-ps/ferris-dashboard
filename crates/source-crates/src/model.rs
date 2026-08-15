//! crates.io API response models (source-specific).

use serde::Deserialize;

/// `GET /api/v1/crates?sort=recent-downloads` response.
#[derive(Debug, Deserialize)]
pub struct CratesResponse {
    pub crates: Vec<CrateSummary>,
    pub meta: Meta,
}

#[derive(Debug, Deserialize)]
pub struct Meta {
    pub total: u64,
}

/// One crate summary from the list endpoint.
#[derive(Debug, Deserialize)]
pub struct CrateSummary {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub downloads: u64,
    pub recent_downloads: u64,
    pub max_version: String,
    pub newest_version: String,
    pub updated_at: Option<String>,
    pub created_at: Option<String>,
}
