//! Normalized event repository.
//!
//! Ingestion is idempotent: upserting the same `(source, source_id)` twice
//! must not create duplicates (AGENTS.md §41, §42).

use chrono::{DateTime, Utc};
use ferris_core::{EcosystemEvent, EcosystemEventId, EventProvenance, EventType, Source, SourceId};
use sqlx::FromRow;
use sqlx::postgres::PgPool;

/// Persistence for normalized ecosystem events.
#[derive(Debug, Clone)]
pub struct EventRepo {
    pool: PgPool,
}

impl EventRepo {
    /// Create a repository over the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Upsert one event. Returns `true` when a new row was inserted and
    /// `false` when the event already existed (idempotent retry).
    pub async fn upsert(&self, event: &EcosystemEvent) -> Result<bool, crate::DbError> {
        let inserted: bool = sqlx::query_scalar(
            r#"
            INSERT INTO events (id, source, source_id, event_type, title, url, summary, published_at, fetched_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (source, source_id) DO UPDATE
                SET fetched_at = EXCLUDED.fetched_at
            RETURNING (xmax = 0)
            "#,
        )
        .bind(event.id.0)
        .bind(event.source.as_str())
        .bind(&event.source_id)
        .bind(event.event_type.as_str())
        .bind(&event.title)
        .bind(&event.url)
        .bind(&event.summary)
        .bind(event.published_at)
        .bind(event.fetched_at())
        .fetch_one(&self.pool)
        .await?;

        for p in &event.provenance {
            self.upsert_provenance(event.id, p).await?;
        }
        Ok(inserted)
    }

    async fn upsert_provenance(
        &self,
        event_id: EcosystemEventId,
        p: &EventProvenance,
    ) -> Result<(), crate::DbError> {
        sqlx::query(
            r#"
            INSERT INTO event_provenance (event_id, source, source_id, source_url, published_at, fetched_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (event_id, source, source_id) DO NOTHING
            "#,
        )
        .bind(event_id.0)
        .bind(p.source.as_str())
        .bind(&p.source_id.0)
        .bind(&p.source_url)
        .bind(p.published_at)
        .bind(p.fetched_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Latest events, newest first, optionally filtered by event type.
    pub async fn list_latest(
        &self,
        limit: i64,
        event_type: Option<EventType>,
    ) -> Result<Vec<EcosystemEvent>, crate::DbError> {
        let rows: Vec<EventRow> = match event_type {
            Some(t) => {
                sqlx::query_as(
                    r#"
                    SELECT id, source, event_type, title, url, summary, published_at, fetched_at, source_id
                    FROM events
                    WHERE event_type = $1
                    ORDER BY published_at DESC NULLS LAST, created_at DESC
                    LIMIT $2
                    "#,
                )
                .bind(t.as_str())
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as(
                    r#"
                    SELECT id, source, event_type, title, url, summary, published_at, fetched_at, source_id
                    FROM events
                    ORDER BY published_at DESC NULLS LAST, created_at DESC
                    LIMIT $1
                    "#,
                )
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
        };

        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            let provenance = self.provenance_for(row.id).await?;
            events.push(row.into_domain(provenance));
        }
        Ok(events)
    }

    /// Find a single event by its database id.
    pub async fn find_by_id(
        &self,
        id: EcosystemEventId,
    ) -> Result<Option<EcosystemEvent>, crate::DbError> {
        let row: Option<EventRow> = sqlx::query_as(
            r#"
            SELECT id, source, event_type, title, url, summary, published_at, fetched_at, source_id
            FROM events
            WHERE id = $1
            "#,
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let provenance = self.provenance_for(row.id).await?;
                Ok(Some(row.into_domain(provenance)))
            }
            None => Ok(None),
        }
    }

    async fn provenance_for(
        &self,
        event_id: uuid::Uuid,
    ) -> Result<Vec<EventProvenance>, crate::DbError> {
        let rows: Vec<ProvenanceRow> = sqlx::query_as(
            r#"
            SELECT source, source_id, source_url, published_at, fetched_at
            FROM event_provenance
            WHERE event_id = $1
            ORDER BY fetched_at
            "#,
        )
        .bind(event_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into_domain()).collect())
    }

    /// Total number of events (used by the overview section).
    pub async fn count(&self) -> Result<i64, crate::DbError> {
        Ok(sqlx::query_scalar("SELECT count(*) FROM events")
            .fetch_one(&self.pool)
            .await?)
    }

    /// Number of events ingested in the last 24 hours.
    pub async fn count_recent(&self, since: DateTime<Utc>) -> Result<i64, crate::DbError> {
        Ok(
            sqlx::query_scalar("SELECT count(*) FROM events WHERE fetched_at >= $1")
                .bind(since)
                .fetch_one(&self.pool)
                .await?,
        )
    }
}

#[derive(FromRow)]
struct EventRow {
    id: uuid::Uuid,
    source: String,
    event_type: String,
    title: String,
    url: String,
    summary: Option<String>,
    published_at: Option<DateTime<Utc>>,
    /// Kept for future retention/audit queries; not surfaced in the domain.
    #[allow(dead_code)]
    fetched_at: DateTime<Utc>,
    source_id: String,
}

impl EventRow {
    fn into_domain(self, provenance: Vec<EventProvenance>) -> EcosystemEvent {
        let source = parse_source(&self.source);
        EcosystemEvent {
            id: EcosystemEventId(self.id),
            source,
            event_type: parse_event_type(&self.event_type),
            title: self.title,
            url: self.url,
            summary: self.summary,
            published_at: self.published_at,
            source_id: self.source_id,
            provenance,
        }
    }
}

#[derive(FromRow)]
struct ProvenanceRow {
    source: String,
    source_id: String,
    source_url: String,
    published_at: Option<DateTime<Utc>>,
    fetched_at: DateTime<Utc>,
}

impl ProvenanceRow {
    fn into_domain(self) -> EventProvenance {
        EventProvenance {
            source: parse_source(&self.source),
            source_id: SourceId(self.source_id),
            source_url: self.source_url,
            published_at: self.published_at,
            fetched_at: self.fetched_at,
        }
    }
}

fn parse_source(s: &str) -> Source {
    match s {
        "rust_blog" => Source::RustBlog,
        "twir" => Source::Twir,
        "crates" => Source::Crates,
        "github" => Source::Github,
        "rustsec" => Source::Rustsec,
        "rfc" => Source::Rfc,
        other => {
            tracing::warn!(source = other, "unknown source string in database");
            Source::Other
        }
    }
}

fn parse_event_type(s: &str) -> EventType {
    match s {
        "release" => EventType::Release,
        "security" => EventType::Security,
        "language" => EventType::Language,
        "compiler" => EventType::Compiler,
        "library" => EventType::Library,
        "framework" => EventType::Framework,
        "tooling" => EventType::Tooling,
        "project" => EventType::Project,
        "community" => EventType::Community,
        "conference" => EventType::Conference,
        "adoption" => EventType::Adoption,
        "performance" => EventType::Performance,
        "funding" => EventType::Funding,
        "job" => EventType::Job,
        _ => EventType::Other,
    }
}
