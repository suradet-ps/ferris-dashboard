//! Project and crate repositories (GitHub + crates.io observations).

use chrono::{DateTime, Utc};
use ferris_core::{Crate, CrateId, Project, ProjectId};
use sqlx::FromRow;
use sqlx::postgres::PgPool;

/// Persistence for GitHub project snapshots.
#[derive(Debug, Clone)]
pub struct ProjectRepo {
    pool: PgPool,
}

impl ProjectRepo {
    /// Create a repository over the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Upsert the latest snapshot of a project (idempotent).
    pub async fn upsert(&self, project: &Project) -> Result<(), crate::DbError> {
        sqlx::query(
            r#"
            INSERT INTO projects (id, name, description, html_url, stars, forks, open_issues, language, archived, pushed_at, fetched_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (id) DO UPDATE SET
                description = EXCLUDED.description,
                stars = EXCLUDED.stars,
                forks = EXCLUDED.forks,
                open_issues = EXCLUDED.open_issues,
                language = EXCLUDED.language,
                archived = EXCLUDED.archived,
                pushed_at = EXCLUDED.pushed_at,
                fetched_at = EXCLUDED.fetched_at
            "#,
        )
        .bind(&project.id.0)
        .bind(&project.name)
        .bind(&project.description)
        .bind(&project.html_url)
        .bind(project.stars as i64)
        .bind(project.forks as i64)
        .bind(project.open_issues as i64)
        .bind(&project.language)
        .bind(project.archived)
        .bind(project.pushed_at)
        .bind(project.fetched_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Projects ordered by star count, for the "top projects" section.
    pub async fn top_by_stars(&self, limit: i64) -> Result<Vec<Project>, crate::DbError> {
        let rows: Vec<ProjectRow> = sqlx::query_as(
            r#"
            SELECT id, name, description, html_url, stars, forks, open_issues, language, archived, pushed_at, fetched_at
            FROM projects
            WHERE archived = false
            ORDER BY stars DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into_domain()).collect())
    }

    /// Find a single project by its canonical id.
    pub async fn find(&self, id: &ProjectId) -> Result<Option<Project>, crate::DbError> {
        let row: Option<ProjectRow> = sqlx::query_as(
            r#"
            SELECT id, name, description, html_url, stars, forks, open_issues, language, archived, pushed_at, fetched_at
            FROM projects
            WHERE id = $1
            "#,
        )
        .bind(&id.0)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into_domain()))
    }

    /// Number of tracked projects.
    pub async fn count(&self) -> Result<i64, crate::DbError> {
        Ok(sqlx::query_scalar("SELECT count(*) FROM projects")
            .fetch_one(&self.pool)
            .await?)
    }
}

/// Persistence for crates.io snapshots.
#[derive(Debug, Clone)]
pub struct CrateRepo {
    pool: PgPool,
}

impl CrateRepo {
    /// Create a repository over the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Upsert the latest snapshot of a crate (idempotent).
    pub async fn upsert(&self, krate: &Crate) -> Result<(), crate::DbError> {
        sqlx::query(
            r#"
            INSERT INTO crates (id, name, description, homepage, repository, downloads, recent_downloads, max_version, updated_at, fetched_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (id) DO UPDATE SET
                description = EXCLUDED.description,
                homepage = EXCLUDED.homepage,
                repository = EXCLUDED.repository,
                downloads = EXCLUDED.downloads,
                recent_downloads = EXCLUDED.recent_downloads,
                max_version = EXCLUDED.max_version,
                updated_at = EXCLUDED.updated_at,
                fetched_at = EXCLUDED.fetched_at
            "#,
        )
        .bind(&krate.id.0)
        .bind(&krate.name)
        .bind(&krate.description)
        .bind(&krate.homepage)
        .bind(&krate.repository)
        .bind(krate.downloads as i64)
        .bind(krate.recent_downloads as i64)
        .bind(&krate.max_version)
        .bind(krate.updated_at)
        .bind(krate.fetched_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Crates ordered by recent downloads (velocity proxy).
    pub async fn top_by_recent_downloads(&self, limit: i64) -> Result<Vec<Crate>, crate::DbError> {
        let rows: Vec<CrateRow> = sqlx::query_as(
            r#"
            SELECT id, name, description, homepage, repository, downloads, recent_downloads, max_version, updated_at, fetched_at
            FROM crates
            ORDER BY recent_downloads DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into_domain()).collect())
    }

    /// Number of tracked crates.
    pub async fn count(&self) -> Result<i64, crate::DbError> {
        Ok(sqlx::query_scalar("SELECT count(*) FROM crates")
            .fetch_one(&self.pool)
            .await?)
    }
}

#[derive(FromRow)]
struct ProjectRow {
    id: String,
    name: String,
    description: Option<String>,
    html_url: String,
    stars: i64,
    forks: i64,
    open_issues: i64,
    language: Option<String>,
    archived: bool,
    pushed_at: Option<DateTime<Utc>>,
    fetched_at: DateTime<Utc>,
}

impl ProjectRow {
    fn into_domain(self) -> Project {
        Project {
            id: ProjectId(self.id),
            name: self.name,
            description: self.description,
            html_url: self.html_url,
            stars: self.stars as u64,
            forks: self.forks as u64,
            open_issues: self.open_issues as u64,
            language: self.language,
            archived: self.archived,
            pushed_at: self.pushed_at,
            fetched_at: self.fetched_at,
        }
    }
}

#[derive(FromRow)]
struct CrateRow {
    id: String,
    name: String,
    description: Option<String>,
    homepage: Option<String>,
    repository: Option<String>,
    downloads: i64,
    recent_downloads: i64,
    max_version: String,
    updated_at: Option<DateTime<Utc>>,
    fetched_at: DateTime<Utc>,
}

impl CrateRow {
    fn into_domain(self) -> Crate {
        Crate {
            id: CrateId(self.id),
            name: self.name,
            description: self.description,
            homepage: self.homepage,
            repository: self.repository,
            downloads: self.downloads as u64,
            recent_downloads: self.recent_downloads as u64,
            max_version: self.max_version,
            updated_at: self.updated_at,
            fetched_at: self.fetched_at,
        }
    }
}
