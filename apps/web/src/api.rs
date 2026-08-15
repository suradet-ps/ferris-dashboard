//! Domain-oriented API (AGENTS.md §39).
//!
//! The frontend consumes only these server functions; it never talks to
//! PostgreSQL or external sources directly.

use ferris_core::{Crate, EcosystemEvent, Project, ScoreBreakdown, SecurityAdvisory, SourceHealth};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use ferris_core::{EventType, Source, SourceStatus};

/// Everything the overview page needs in one response (no API waterfall).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub latest_events: Vec<EcosystemEvent>,
    pub trending: Vec<TrendingProject>,
    pub top_crates: Vec<Crate>,
    pub top_projects: Vec<Project>,
    pub recent_advisories: Vec<SecurityAdvisory>,
    pub health: Vec<SourceHealth>,
    pub stats: OverviewStats,
}

/// Aggregate counters for the overview strip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverviewStats {
    pub events_total: i64,
    pub events_24h: i64,
    pub projects: i64,
    pub crates: i64,
    pub advisories: i64,
    pub healthy_sources: usize,
    pub degraded_sources: usize,
}

/// A project with its explainable momentum score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingProject {
    pub project: Project,
    pub score: ScoreBreakdown,
}

/// Load the full dashboard payload.
#[server(prefix = "/api", endpoint = "dashboard")]
pub async fn get_dashboard() -> Result<DashboardData, ServerFnError> {
    let pool = db_pool()?;
    let events = ferris_database::EventRepo::new(pool.clone());
    let projects = ferris_database::ProjectRepo::new(pool.clone());
    let crates = ferris_database::CrateRepo::new(pool.clone());
    let metrics = ferris_database::MetricRepo::new(pool.clone());
    let advisories = ferris_database::AdvisoryRepo::new(pool.clone());
    let health = ferris_database::SourceStatusRepo::new(pool.clone());

    let now = chrono::Utc::now();
    let day_ago = now - chrono::Duration::days(1);

    let (latest_events, top_projects, top_crates, recent_advisories) = tokio::join!(
        events.list_latest(20, None),
        projects.top_by_stars(20),
        crates.top_by_recent_downloads(20),
        advisories.list_recent(10),
    );

    let trending = trending_projects(&projects, &metrics, 10).await?;
    let source_health = health.all(&interval_defaults()).await?;

    let (events_total, events_24h, project_count, crate_count, advisory_count) = (
        events.count().await?,
        events.count_recent(day_ago).await?,
        projects.count().await?,
        crates.count().await?,
        advisories.count().await?,
    );

    let stats = OverviewStats {
        events_total,
        events_24h,
        projects: project_count,
        crates: crate_count,
        advisories: advisory_count,
        healthy_sources: source_health
            .iter()
            .filter(|h| h.status == SourceStatus::Healthy)
            .count(),
        degraded_sources: source_health
            .iter()
            .filter(|h| h.status == SourceStatus::Degraded)
            .count(),
    };

    Ok(DashboardData {
        latest_events: latest_events.unwrap_or_default(),
        trending,
        top_crates: top_crates.unwrap_or_default(),
        top_projects: top_projects.unwrap_or_default(),
        recent_advisories: recent_advisories.unwrap_or_default(),
        health: source_health,
        stats,
    })
}

/// Latest events, optionally filtered by event type.
#[server(prefix = "/api", endpoint = "events")]
pub async fn get_events(event_type: Option<String>) -> Result<Vec<EcosystemEvent>, ServerFnError> {
    let pool = db_pool()?;
    let repo = ferris_database::EventRepo::new(pool);
    let filter =
        event_type.and_then(|t| serde_json::from_str::<EventType>(&format!("\"{t}\"")).ok());
    Ok(repo.list_latest(100, filter).await?)
}

/// Trending projects with explainable momentum.
#[server(prefix = "/api", endpoint = "trending")]
pub async fn get_trending() -> Result<Vec<TrendingProject>, ServerFnError> {
    let pool = db_pool()?;
    let projects = ferris_database::ProjectRepo::new(pool.clone());
    let metrics = ferris_database::MetricRepo::new(pool);
    trending_projects(&projects, &metrics, 25).await
}

/// Security advisories.
#[server(prefix = "/api", endpoint = "security")]
pub async fn get_security() -> Result<Vec<SecurityAdvisory>, ServerFnError> {
    let pool = db_pool()?;
    Ok(ferris_database::AdvisoryRepo::new(pool)
        .list_recent(100)
        .await?)
}

/// Source freshness + health.
#[server(prefix = "/api", endpoint = "sources")]
pub async fn get_source_health() -> Result<Vec<SourceHealth>, ServerFnError> {
    let pool = db_pool()?;
    Ok(ferris_database::SourceStatusRepo::new(pool)
        .all(&interval_defaults())
        .await?)
}

/// Top crates by recent downloads.
#[server(prefix = "/api", endpoint = "crates")]
pub async fn get_top_crates(limit: i64) -> Result<Vec<Crate>, ServerFnError> {
    let pool = db_pool()?;
    Ok(ferris_database::CrateRepo::new(pool)
        .top_by_recent_downloads(limit)
        .await?)
}

/// Compute momentum scores for the top projects.
#[cfg(feature = "ssr")]
async fn trending_projects(
    repo: &ferris_database::ProjectRepo,
    metrics: &ferris_database::MetricRepo,
    limit: i64,
) -> Result<Vec<TrendingProject>, ServerFnError> {
    let projects = repo.top_by_stars(limit).await?;
    let mut out = Vec::with_capacity(projects.len());
    for project in projects {
        let series = metrics
            .series(ferris_database::EntityKind::Project, &project.id.0, "stars")
            .await?;
        let input = ferris_scoring::MomentumInput {
            stars: series
                .iter()
                .map(|p| (p.timestamp, p.value as u64))
                .collect(),
            downloads: Vec::new(),
            contributors: Vec::new(),
            releases: Vec::new(),
            community: Vec::new(),
        };
        let score = ferris_scoring::momentum_for_project(&project.id, &input);
        out.push(TrendingProject { project, score });
    }
    // Highest momentum first.
    out.sort_by(|a, b| {
        b.score
            .total
            .partial_cmp(&a.score.total)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(out)
}

/// Resolve the database pool from Leptos context, or report it offline.
#[cfg(feature = "ssr")]
fn db_pool() -> Result<ferris_database::DbPool, ServerFnError> {
    use_context::<Option<ferris_database::DbPool>>()
        .flatten()
        .ok_or_else(|| {
            ServerFnError::ServerError(
                "database unavailable: set DATABASE_URL and start the ingestion worker".into(),
            )
        })
}

/// (source, interval_minutes) pairs for the health endpoint.
#[cfg(feature = "ssr")]
fn interval_defaults() -> Vec<(Source, u64)> {
    Source::ALL
        .iter()
        .map(|s| (*s, s.default_interval_minutes()))
        .collect()
}
