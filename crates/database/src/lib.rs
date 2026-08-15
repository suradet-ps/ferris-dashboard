//! SQLx repository layer over PostgreSQL.
//!
//! SQL stays close to this layer (AGENTS.md §20): UI components and source
//! connectors never contain SQL, and repositories never leak into the domain.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod advisories;
pub mod events;
pub mod metrics;
pub mod projects;
pub mod source_status;

pub use advisories::AdvisoryRepo;
pub use events::EventRepo;
pub use metrics::{EntityKind, MetricRepo};
pub use projects::{CrateRepo, ProjectRepo};
pub use source_status::SourceStatusRepo;

use sqlx::postgres::PgPoolOptions;
use sqlx::{Error, PgPool};

pub use sqlx::PgPool as DbPool;

/// Errors produced by the database layer.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// A SQLx-level failure (connection, constraint, type mapping).
    #[error("database error: {0}")]
    Sqlx(#[from] Error),
    /// A migration failure.
    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

/// Connect to PostgreSQL at `database_url` and run pending migrations.
pub async fn connect_and_migrate(database_url: &str) -> Result<PgPool, DbError> {
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(database_url)
        .await?;
    sqlx::migrate!("../../migrations").run(&pool).await?;
    Ok(pool)
}
