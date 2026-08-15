//! Time-series metric repository.
//!
//! Stores observations (timestamp, value) per entity and metric so velocity
//! and trend detection are possible (AGENTS.md §19).

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use sqlx::postgres::PgPool;

/// Type of entity a metric observation belongs to.
#[derive(Debug, Clone, Copy)]
pub enum EntityKind {
    /// Observations about a GitHub project.
    Project,
    /// Observations about a crates.io crate.
    Crate,
}

impl EntityKind {
    fn as_str(&self) -> &'static str {
        match self {
            EntityKind::Project => "project",
            EntityKind::Crate => "crate",
        }
    }
}

/// Persistence for historical metric observations.
#[derive(Debug, Clone)]
pub struct MetricRepo {
    pool: PgPool,
}

impl MetricRepo {
    /// Create a repository over the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Record one observation. Duplicates on `(entity, metric, timestamp)`
    /// are ignored (idempotent).
    pub async fn record(
        &self,
        kind: EntityKind,
        entity_id: &str,
        metric: &str,
        value: f64,
        observed_at: DateTime<Utc>,
    ) -> Result<(), crate::DbError> {
        sqlx::query(
            r#"
            INSERT INTO metrics (entity_type, entity_id, metric, value, observed_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (entity_type, entity_id, metric, observed_at) DO NOTHING
            "#,
        )
        .bind(kind.as_str())
        .bind(entity_id)
        .bind(metric)
        .bind(value)
        .bind(observed_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// All observations for one entity+metric, oldest first.
    pub async fn series(
        &self,
        kind: EntityKind,
        entity_id: &str,
        metric: &str,
    ) -> Result<Vec<MetricPoint>, crate::DbError> {
        let rows: Vec<MetricRow> = sqlx::query_as(
            r#"
            SELECT observed_at, value
            FROM metrics
            WHERE entity_type = $1 AND entity_id = $2 AND metric = $3
            ORDER BY observed_at
            "#,
        )
        .bind(kind.as_str())
        .bind(entity_id)
        .bind(metric)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| MetricPoint {
                timestamp: r.observed_at,
                value: r.value,
            })
            .collect())
    }
}

/// One historical observation.
#[derive(Debug, Clone, Copy)]
pub struct MetricPoint {
    /// When the observation was recorded.
    pub timestamp: DateTime<Utc>,
    /// The observed value.
    pub value: f64,
}

#[derive(FromRow)]
struct MetricRow {
    observed_at: DateTime<Utc>,
    value: f64,
}
