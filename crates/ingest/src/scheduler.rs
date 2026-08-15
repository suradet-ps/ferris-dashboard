//! Source scheduler.
//!
//! Each source runs on its own configurable interval (AGENTS.md §12) and is
//! executed independently, in parallel. A failing source never blocks or
//! aborts the others.

use crate::connector::SourceConnector;
use crate::pipeline::Pipeline;
use ferris_core::Source;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{info, warn};

const TICK_MS: u64 = 15_000;

/// Runs connectors on independent configurable intervals.
#[derive(Debug)]
pub struct Scheduler {
    pipeline: Arc<Pipeline>,
    connectors: Vec<Box<dyn SourceConnector>>,
    intervals: HashMap<Source, Duration>,
    last_run: HashMap<Source, Instant>,
}

impl Scheduler {
    /// Build a scheduler. Default intervals come from
    /// [`Source::default_interval_minutes`] (AGENTS.md §12) and can be
    /// overridden per source with [`Scheduler::with_interval`].
    pub fn new(pipeline: Pipeline, connectors: Vec<Box<dyn SourceConnector>>) -> Self {
        let mut intervals = HashMap::new();
        for source in Source::ALL {
            intervals.insert(
                source,
                Duration::from_secs(source.default_interval_minutes() * 60),
            );
        }
        Self {
            pipeline: Arc::new(pipeline),
            connectors,
            intervals,
            last_run: HashMap::new(),
        }
    }

    /// Override the interval for one source.
    pub fn with_interval(mut self, source: Source, interval: Duration) -> Self {
        self.intervals.insert(source, interval);
        self
    }

    /// Interval configured for a source, in minutes.
    pub fn interval_minutes(&self, source: Source) -> u64 {
        self.intervals
            .get(&source)
            .copied()
            .unwrap_or(Duration::from_secs(3600))
            .as_secs()
            / 60
    }

    /// Run forever: every tick, run each connector whose interval elapsed.
    ///
    /// Set the watch value to `true` to stop gracefully (e.g. on Ctrl-C).
    pub async fn run(&mut self, mut shutdown: tokio::sync::watch::Receiver<bool>) {
        let mut ticker = tokio::time::interval(Duration::from_millis(TICK_MS));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    let due: Vec<Source> = self
                        .connectors
                        .iter()
                        .filter(|c| self.is_due(c.source()))
                        .map(|c| c.source())
                        .collect();
                    if !due.is_empty() {
                        self.run_due(&due).await;
                    }
                }
                changed = shutdown.changed() => {
                    if changed.is_ok() && *shutdown.borrow() {
                        info!("shutdown signal received, stopping scheduler");
                        return;
                    }
                }
            }
        }
    }

    fn is_due(&self, source: Source) -> bool {
        match self.last_run.get(&source) {
            None => true,
            Some(last) => {
                let interval = self
                    .intervals
                    .get(&source)
                    .copied()
                    .unwrap_or(Duration::from_secs(3600));
                last.elapsed() >= interval
            }
        }
    }

    async fn run_due(&mut self, sources: &[Source]) {
        // All connectors run concurrently, isolated from each other: one
        // failing connector cannot abort the rest (AGENTS.md §10).
        let pipeline = self.pipeline.clone();
        let tasks: Vec<_> = sources
            .iter()
            .map(|source| {
                let connector: &dyn SourceConnector = self
                    .connectors
                    .iter()
                    .find(|c| c.source() == *source)
                    .map(|b| b.as_ref())
                    .expect("connector registered for its own source");
                let source = *source;
                let pipeline = pipeline.clone();
                async move {
                    let result = pipeline.run_source(connector).await;
                    (source, result)
                }
            })
            .collect();

        let results = futures::future::join_all(tasks).await;
        for (source, result) in results {
            self.last_run.insert(source, Instant::now());
            match result {
                Ok(summary) => info!(
                    source = %summary.source,
                    items = summary.items,
                    new = summary.new_events,
                    duplicates = summary.duplicate_events,
                    duration_ms = summary.duration.as_millis(),
                    "source.run.completed"
                ),
                Err(err) => warn!(source = %source, error = %err, "source.run.failed"),
            }
        }
    }
}
