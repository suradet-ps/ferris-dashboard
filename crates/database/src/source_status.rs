//! Source health / freshness repository.

use chrono::{DateTime, Utc};
use ferris_core::{Source, SourceHealth, SourceStatus};
use sqlx::FromRow;
use sqlx::postgres::PgPool;

/// Persistence for per-source fetch health.
#[derive(Debug, Clone)]
pub struct SourceStatusRepo {
    pool: PgPool,
}

impl SourceStatusRepo {
    /// Create a repository over the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Record a successful fetch.
    pub async fn record_success(
        &self,
        source: Source,
        duration_ms: u64,
    ) -> Result<(), crate::DbError> {
        sqlx::query(
            r#"
            INSERT INTO source_health (source, status, last_fetched_at, last_attempted_at, last_duration_ms, last_error, updated_at)
            VALUES ($1, 'healthy', $2, $2, $3, NULL, $2)
            ON CONFLICT (source) DO UPDATE SET
                status = 'healthy',
                last_fetched_at = EXCLUDED.last_fetched_at,
                last_attempted_at = EXCLUDED.last_attempted_at,
                last_duration_ms = EXCLUDED.last_duration_ms,
                last_error = NULL,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(source.as_str())
        .bind(Utc::now())
        .bind(duration_ms as i64)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Record a failed fetch. The source becomes `degraded`; retries use
    /// exponential backoff driven by `last_attempted_at`.
    pub async fn record_failure(&self, source: Source, error: &str) -> Result<(), crate::DbError> {
        sqlx::query(
            r#"
            INSERT INTO source_health (source, status, last_attempted_at, last_error, updated_at)
            VALUES ($1, 'degraded', $2, $3, $2)
            ON CONFLICT (source) DO UPDATE SET
                status = 'degraded',
                last_attempted_at = EXCLUDED.last_attempted_at,
                last_error = EXCLUDED.last_error,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(source.as_str())
        .bind(Utc::now())
        .bind(error)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Health of all sources, defaulting to `disabled` when absent.
    pub async fn all(
        &self,
        interval_minutes: &[(Source, u64)],
    ) -> Result<Vec<SourceHealth>, crate::DbError> {
        let rows: Vec<HealthRow> = sqlx::query_as(
            r#"
            SELECT source, status, last_fetched_at, last_attempted_at, last_error, last_duration_ms
            FROM source_health
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut by_source: std::collections::HashMap<String, HealthRow> =
            rows.into_iter().map(|r| (r.source.clone(), r)).collect();

        let mut health = Vec::with_capacity(interval_minutes.len());
        for (source, minutes) in interval_minutes {
            let row = by_source.remove(source.as_str());
            health.push(match row {
                Some(row) => row.into_domain(*source, *minutes),
                None => SourceHealth {
                    source: *source,
                    status: SourceStatus::Disabled,
                    last_fetched_at: None,
                    last_attempted_at: None,
                    last_error: None,
                    last_duration_ms: None,
                    interval_minutes: *minutes,
                },
            });
        }
        Ok(health)
    }
}

#[derive(FromRow)]
struct HealthRow {
    source: String,
    status: String,
    last_fetched_at: Option<DateTime<Utc>>,
    last_attempted_at: Option<DateTime<Utc>>,
    last_error: Option<String>,
    last_duration_ms: Option<i64>,
}

impl HealthRow {
    fn into_domain(self, source: Source, interval_minutes: u64) -> SourceHealth {
        let status = match self.status.as_str() {
            "healthy" => SourceStatus::Healthy,
            "degraded" => SourceStatus::Degraded,
            _ => SourceStatus::Disabled,
        };
        SourceHealth {
            source,
            status,
            last_fetched_at: self.last_fetched_at,
            last_attempted_at: self.last_attempted_at,
            last_error: self.last_error,
            last_duration_ms: self.last_duration_ms.map(|v| v as u64),
            interval_minutes,
        }
    }
}
