//! Ingestion pipeline: fetch → normalize → deduplicate → persist.
//!
//! Source failures are isolated: `run_source` records the failure and
//! returns it, and the scheduler continues with the remaining sources
//! (AGENTS.md §10).

use crate::connector::{NormalizedOutput, SourceConnector};
use ferris_core::Source;
use ferris_database::{
    AdvisoryRepo, CrateRepo, DbError, EventRepo, MetricRepo, ProjectRepo, SourceStatusRepo,
};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Repository bundle used by the pipeline.
#[derive(Debug, Clone)]
pub struct Pipeline {
    /// Normalized event persistence (dedup via `(source, source_id)`).
    pub events: EventRepo,
    /// GitHub project snapshot persistence.
    pub projects: ProjectRepo,
    /// crates.io crate snapshot persistence.
    pub crates: CrateRepo,
    /// Time-series metric observations for momentum.
    pub metrics: MetricRepo,
    /// Security advisory persistence.
    pub advisories: AdvisoryRepo,
    /// Per-source fetch health (freshness reporting).
    pub health: SourceStatusRepo,
}

/// Result of one connector run.
#[derive(Debug, Clone)]
pub struct RunSummary {
    /// The source that ran.
    pub source: Source,
    /// When the run finished.
    pub fetched_at: chrono::DateTime<chrono::Utc>,
    /// Number of normalized items produced.
    pub items: usize,
    /// Events inserted for the first time.
    pub new_events: usize,
    /// Events that already existed (idempotent retry).
    pub duplicate_events: usize,
    /// Wall-clock duration of the run.
    pub duration: Duration,
}

impl Pipeline {
    /// Run one connector end-to-end: fetch, persist, record health.
    ///
    /// Returns `Ok(summary)` on success and `Err` with the connector error
    /// on failure. Either way, source health is recorded first.
    pub async fn run_source(
        &self,
        connector: &dyn SourceConnector,
    ) -> Result<RunSummary, crate::ConnectorError> {
        let source = connector.source();
        let started = Instant::now();

        let outputs = match connector.fetch().await {
            Ok(outputs) => outputs,
            Err(err) => {
                if let Err(db_err) = self.health.record_failure(source, &err.to_string()).await {
                    error!(source = %source, error = %db_err, "failed to record source failure");
                }
                error!(source = %source, error = %err, "source.fetch.failed");
                return Err(err);
            }
        };

        info!(source = %source, items = outputs.len(), "source.fetch.completed");

        let mut new_events = 0usize;
        let mut duplicate_events = 0usize;

        for output in &outputs {
            match self.persist(output).await {
                Ok(Some(true)) => new_events += 1,
                Ok(Some(false)) => duplicate_events += 1,
                Ok(None) => {}
                Err(db_err) => {
                    warn!(source = %source, error = %db_err, "database.write.failed");
                }
            }
        }

        if let Err(db_err) = self
            .health
            .record_success(source, started.elapsed().as_millis() as u64)
            .await
        {
            error!(source = %source, error = %db_err, "failed to record source success");
        }

        Ok(RunSummary {
            source,
            fetched_at: chrono::Utc::now(),
            items: outputs.len(),
            new_events,
            duplicate_events,
            duration: started.elapsed(),
        })
    }

    /// Persist one normalized output idempotently.
    ///
    /// Returns:
    /// - `Ok(Some(true))` when a new row was inserted,
    /// - `Ok(Some(false))` when the row already existed (deduplicated),
    /// - `Ok(None)` for outputs with no dedup semantics (snapshots).
    async fn persist(&self, output: &NormalizedOutput) -> Result<Option<bool>, DbError> {
        match output {
            NormalizedOutput::Event(event) => {
                debug!(event = %event.title, "event.normalized");
                Ok(Some(self.events.upsert(event).await?))
            }
            NormalizedOutput::Project(project) => {
                self.projects.upsert(project).await?;
                // Record a historical star observation for momentum.
                self.metrics
                    .record(
                        ferris_database::EntityKind::Project,
                        &project.id.0,
                        "stars",
                        project.stars as f64,
                        project.fetched_at,
                    )
                    .await?;
                Ok(None)
            }
            NormalizedOutput::Crate(krate) => {
                self.crates.upsert(krate).await?;
                self.metrics
                    .record(
                        ferris_database::EntityKind::Crate,
                        &krate.id.0,
                        "recent_downloads",
                        krate.recent_downloads as f64,
                        krate.fetched_at,
                    )
                    .await?;
                Ok(None)
            }
            NormalizedOutput::Advisory(advisory) => {
                Ok(Some(self.advisories.upsert(advisory).await?))
            }
        }
    }
}
