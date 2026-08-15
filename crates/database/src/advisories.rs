//! Security advisory repository (RustSec).

use chrono::{DateTime, Utc};
use ferris_core::SecurityAdvisory;
use sqlx::FromRow;
use sqlx::postgres::PgPool;
use sqlx::types::Json;

/// Persistence for normalized security advisories.
#[derive(Debug, Clone)]
pub struct AdvisoryRepo {
    pool: PgPool,
}

impl AdvisoryRepo {
    /// Create a repository over the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Upsert one advisory (idempotent on the advisory id).
    pub async fn upsert(&self, advisory: &SecurityAdvisory) -> Result<bool, crate::DbError> {
        let inserted: bool = sqlx::query_scalar(
            r#"
            INSERT INTO advisories (id, title, url, summary, affected_crates, published_at, fetched_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
                fetched_at = EXCLUDED.fetched_at
            RETURNING (xmax = 0)
            "#,
        )
        .bind(&advisory.id)
        .bind(&advisory.title)
        .bind(&advisory.url)
        .bind(&advisory.summary)
        .bind(Json(&advisory.affected_crates))
        .bind(advisory.published_at)
        .bind(advisory.fetched_at)
        .fetch_one(&self.pool)
        .await?;
        Ok(inserted)
    }

    /// Most recent advisories, newest first.
    pub async fn list_recent(&self, limit: i64) -> Result<Vec<SecurityAdvisory>, crate::DbError> {
        let rows: Vec<AdvisoryRow> = sqlx::query_as(
            r#"
            SELECT id, title, url, summary, affected_crates, published_at, fetched_at
            FROM advisories
            ORDER BY published_at DESC NULLS LAST, fetched_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into_domain()).collect())
    }

    /// Total advisory count (overview section).
    pub async fn count(&self) -> Result<i64, crate::DbError> {
        Ok(sqlx::query_scalar("SELECT count(*) FROM advisories")
            .fetch_one(&self.pool)
            .await?)
    }
}

#[derive(FromRow)]
struct AdvisoryRow {
    id: String,
    title: String,
    url: String,
    summary: Option<String>,
    affected_crates: Json<Vec<String>>,
    published_at: Option<DateTime<Utc>>,
    fetched_at: DateTime<Utc>,
}

impl AdvisoryRow {
    fn into_domain(self) -> SecurityAdvisory {
        SecurityAdvisory {
            id: self.id,
            title: self.title,
            url: self.url,
            summary: self.summary,
            affected_crates: self.affected_crates.0,
            published_at: self.published_at,
            fetched_at: self.fetched_at,
        }
    }
}
