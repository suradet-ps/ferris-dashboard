//! Domain models and business rules for the Rust ecosystem dashboard.
//!
//! This crate MUST NOT depend on Leptos, Axum, SQLx, Reqwest, PostgreSQL,
//! HTML parsers or RSS parsers. It must remain testable without network,
//! database, filesystem or browser access.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod classification;
pub mod event;
pub mod health;
pub mod id;
pub mod project;
pub mod security;
pub mod source;

pub use classification::classify;
pub use event::{EcosystemEvent, EcosystemEventId, EventType};
pub use health::{SourceHealth, SourceStatus};
pub use id::{CrateId, ProjectId};
pub use project::{Crate, Project, ScoreBreakdown, TimeSeriesPoint};
pub use security::SecurityAdvisory;
pub use source::{EventProvenance, Source, SourceId};
