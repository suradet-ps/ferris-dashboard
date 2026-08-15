//! Strong identifiers for domain concepts.
//!
//! All identifiers are serialized as plain strings over the wire so the
//! API layer stays stable regardless of the underlying representation.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Identifier of a crate on crates.io (the crate `name`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CrateId(pub String);

impl fmt::Display for CrateId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Identifier of a project (canonical `owner/name` for GitHub-hosted projects).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub String);

impl fmt::Display for ProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
