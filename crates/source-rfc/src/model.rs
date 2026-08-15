//! GitHub contents API models for the RFC repo.

use serde::Deserialize;

/// One entry of `GET /repos/rust-lang/rfcs/contents/text`.
#[derive(Debug, Deserialize)]
pub struct ContentsEntry {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub html_url: Option<String>,
    /// "file" for RFC text files.
    #[serde(rename = "type")]
    pub kind: String,
}
