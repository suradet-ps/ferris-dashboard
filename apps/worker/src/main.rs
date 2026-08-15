//! Ingestion worker binary.
//!
//! Reads configuration from the environment, connects to PostgreSQL, builds
//! the connector set, and runs the scheduler until shutdown.

use anyhow::Context;
use ferris_ingest::{Pipeline, Scheduler};
use std::time::Duration;
use tracing::info;
use tracing_subscriber::EnvFilter;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,ferris_worker=debug")),
        )
        .init();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("failed to build tokio runtime")?;
    runtime.block_on(async_main())
}

async fn async_main() -> anyhow::Result<()> {
    let config = config::Config::from_env()?;
    info!(
        database_url_set = !config.database_url.is_empty(),
        "worker starting"
    );

    let pool = ferris_database::connect_and_migrate(&config.database_url)
        .await
        .context("failed to connect to PostgreSQL (set DATABASE_URL)")?;

    let pipeline = Pipeline {
        events: ferris_database::EventRepo::new(pool.clone()),
        projects: ferris_database::ProjectRepo::new(pool.clone()),
        crates: ferris_database::CrateRepo::new(pool.clone()),
        metrics: ferris_database::MetricRepo::new(pool.clone()),
        advisories: ferris_database::AdvisoryRepo::new(pool.clone()),
        health: ferris_database::SourceStatusRepo::new(pool),
    };

    let mut scheduler = Scheduler::new(pipeline, config.connectors()?);

    for &(source, minutes) in config.interval_overrides() {
        scheduler = scheduler.with_interval(source, Duration::from_secs(minutes * 60));
    }

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    install_shutdown_handler(shutdown_tx);

    info!("scheduler running (ctrl-c to stop)");
    scheduler.run(shutdown_rx).await;
    Ok(())
}

fn install_shutdown_handler(tx: tokio::sync::watch::Sender<bool>) {
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        info!("ctrl-c received, shutting down");
        let _ = tx.send(true);
    });
}

mod config {
    use anyhow::Context;
    use ferris_core::Source;
    use ferris_ingest::{HttpFetcher, SourceConnector};
    use std::env;
    use std::fmt;

    /// Worker configuration from the environment (AGENTS.md §43, §44).
    pub struct Config {
        pub database_url: String,
        pub github_token: Option<String>,
        pub github_enabled: bool,
        pub crates_enabled: bool,
        pub rss_enabled: bool,
        pub rustsec_enabled: bool,
        pub rfc_enabled: bool,
        pub interval_overrides: Vec<(Source, u64)>,
    }

    impl Config {
        pub fn from_env() -> anyhow::Result<Self> {
            let database_url = env::var("DATABASE_URL").context("DATABASE_URL is required")?;

            let config = Config {
                database_url,
                github_token: env::var("GITHUB_TOKEN").ok().filter(|s| !s.is_empty()),
                github_enabled: env_flag("SOURCE_GITHUB_ENABLED", true),
                crates_enabled: env_flag("SOURCE_CRATES_ENABLED", true),
                rss_enabled: env_flag("SOURCE_RSS_ENABLED", true),
                rustsec_enabled: env_flag("SOURCE_RUSTSEC_ENABLED", true),
                rfc_enabled: env_flag("SOURCE_RFC_ENABLED", true),
                interval_overrides: vec![
                    env_interval("INGEST_INTERVAL_GITHUB", Source::Github),
                    env_interval("INGEST_INTERVAL_CRATES", Source::Crates),
                    env_interval("INGEST_INTERVAL_RSS", Source::RustBlog),
                    env_interval("INGEST_INTERVAL_RUSTSEC", Source::Rustsec),
                    env_interval("INGEST_INTERVAL_RFC", Source::Rfc),
                ]
                .into_iter()
                .flatten()
                .collect(),
            };
            Ok(config)
        }

        /// Build the configured connector set.
        pub fn connectors(&self) -> anyhow::Result<Vec<Box<dyn SourceConnector>>> {
            let fetcher =
                HttpFetcher::with_bearer_token("ferris-dashboard/0.1", self.github_token.clone())?;

            let mut connectors: Vec<Box<dyn SourceConnector>> = Vec::new();
            if self.rss_enabled {
                connectors.push(Box::new(ferris_source_rss::RssConnector::rust_blog(
                    HttpFetcher::new("ferris-dashboard/0.1")?,
                )));
                connectors.push(Box::new(ferris_source_rss::RssConnector::twir(
                    HttpFetcher::new("ferris-dashboard/0.1")?,
                )));
            }
            if self.github_enabled {
                connectors.push(Box::new(ferris_source_github::GithubConnector::new(
                    fetcher,
                )));
            }
            if self.crates_enabled {
                connectors.push(Box::new(ferris_source_crates::CratesConnector::new(
                    HttpFetcher::new("ferris-dashboard/0.1")?,
                )));
            }
            if self.rustsec_enabled {
                connectors.push(Box::new(ferris_source_rustsec::RustsecConnector::new(
                    HttpFetcher::new("ferris-dashboard/0.1")?,
                )));
            }
            if self.rfc_enabled {
                connectors.push(Box::new(ferris_source_rfc::RfcConnector::new(
                    HttpFetcher::new("ferris-dashboard/0.1")?,
                )));
            }
            Ok(connectors)
        }

        pub fn interval_overrides(&self) -> &[(Source, u64)] {
            &self.interval_overrides
        }
    }

    fn env_flag(name: &str, default: bool) -> bool {
        match env::var(name).ok().as_deref() {
            Some("true" | "1" | "yes") => true,
            Some("false" | "0" | "no") => false,
            _ => default,
        }
    }

    fn env_interval(name: &str, source: Source) -> Option<(Source, u64)> {
        env::var(name)
            .ok()
            .and_then(|v| v.parse().ok())
            .map(|m| (source, m))
    }

    impl fmt::Debug for Config {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Config")
                .field("github_enabled", &self.github_enabled)
                .field("crates_enabled", &self.crates_enabled)
                .field("rss_enabled", &self.rss_enabled)
                .field("rustsec_enabled", &self.rustsec_enabled)
                .field("rfc_enabled", &self.rfc_enabled)
                .field("database_url_set", &(!self.database_url.is_empty()))
                .finish()
        }
    }
}
