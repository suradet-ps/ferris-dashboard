//! Momentum scoring.
//!
//! Conceptually (AGENTS.md §17):
//!
//! ```text
//! Momentum Score = Star Velocity
//!               + Download Velocity
//!               + Contributor Growth
//!               + Release Activity
//!               + Dependency Growth
//!               + Community Signal
//! ```
//!
//! Each component is normalized to `0..=1` with a sigmoid over a reference
//! scale, then combined with explicit weights. All parameters are constants
//! so scores are reproducible and explainable.

use chrono::{DateTime, Utc};
use ferris_core::{CrateId, ProjectId, ScoreBreakdown};

/// Default component weights. They sum to `1.0`.
pub const DEFAULT_WEIGHTS: MomentumWeights = MomentumWeights {
    stars: 0.25,
    downloads: 0.25,
    contributors: 0.2,
    releases: 0.15,
    community: 0.15,
};

/// Sigmoid midpoint (per day) for each component. A component at its
/// midpoint contributes `0.5` to its normalized score.
const STAR_MIDPOINT_PER_DAY: f64 = 50.0;
const DOWNLOAD_MIDPOINT_PER_DAY: f64 = 2_000.0;
const CONTRIBUTOR_MIDPOINT_PER_DAY: f64 = 0.5;
const RELEASE_MIDPOINT_PER_DAY: f64 = 0.02;
const COMMUNITY_MIDPOINT_PER_DAY: f64 = 0.1;

/// Inputs to the momentum calculation for one project.
#[derive(Debug, Default)]
pub struct MomentumInput {
    /// Historical star observations (oldest first).
    pub stars: Vec<(DateTime<Utc>, u64)>,
    /// Historical download observations (oldest first).
    pub downloads: Vec<(DateTime<Utc>, u64)>,
    /// Historical contributor observations (oldest first).
    pub contributors: Vec<(DateTime<Utc>, u64)>,
    /// Historical release observations (oldest first).
    pub releases: Vec<(DateTime<Utc>, u64)>,
    /// Historical community mentions (oldest first).
    pub community: Vec<(DateTime<Utc>, u64)>,
}

/// Explicit, documented weights for the five momentum components.
#[derive(Debug, Clone, Copy)]
pub struct MomentumWeights {
    /// Weight of the star-velocity component.
    pub stars: f64,
    /// Weight of the download-velocity component.
    pub downloads: f64,
    /// Weight of the contributor-growth component.
    pub contributors: f64,
    /// Weight of the release-activity component.
    pub releases: f64,
    /// Weight of the community-signal component.
    pub community: f64,
}

/// Calculate the momentum score for a project.
///
/// # Explainability
///
/// The returned [`ScoreBreakdown`] exposes every component so the UI can
/// answer "why is this trending?" without re-deriving the formula.
pub fn momentum_for_project(id: &ProjectId, input: &MomentumInput) -> ScoreBreakdown {
    let _ = id;
    calculate(input, &DEFAULT_WEIGHTS)
}

/// Calculate the momentum score for a crate (downloads + releases dominate).
pub fn momentum_for_crate(_id: &CrateId, input: &MomentumInput) -> ScoreBreakdown {
    let weights = MomentumWeights {
        stars: 0.05,
        downloads: 0.5,
        contributors: 0.05,
        releases: 0.25,
        community: 0.15,
    };
    calculate(input, &weights)
}

fn calculate(input: &MomentumInput, w: &MomentumWeights) -> ScoreBreakdown {
    let stars = sigmoid_velocity(&input.stars, STAR_MIDPOINT_PER_DAY);
    let downloads = sigmoid_velocity(&input.downloads, DOWNLOAD_MIDPOINT_PER_DAY);
    let contributors = sigmoid_velocity(&input.contributors, CONTRIBUTOR_MIDPOINT_PER_DAY);
    let releases = sigmoid_velocity(&input.releases, RELEASE_MIDPOINT_PER_DAY);
    let community = sigmoid_velocity(&input.community, COMMUNITY_MIDPOINT_PER_DAY);

    let total = w.stars * stars
        + w.downloads * downloads
        + w.contributors * contributors
        + w.releases * releases
        + w.community * community;

    ScoreBreakdown {
        stars,
        downloads,
        contributors,
        releases,
        community,
        total,
    }
}

/// Normalize per-day velocity into `0..=1`.
///
/// Uses a centered sigmoid (`tanh`) so that zero velocity scores exactly
/// `0` (a flat series is NOT momentum) and positive velocity approaches
/// `1` as it grows past the midpoint. Negative velocity (declining
/// popularity) clamps to `0`; momentum is about growth.
fn sigmoid_velocity(points: &[(DateTime<Utc>, u64)], midpoint_per_day: f64) -> f64 {
    if points.len() < 2 {
        return 0.0;
    }
    let pairs: Vec<(DateTime<Utc>, f64)> = points.iter().map(|(ts, v)| (*ts, *v as f64)).collect();
    let per_day = crate::velocity::linear_velocity(&pairs);
    let x = per_day / midpoint_per_day;
    // tanh(x) = 2/(1+e^(-2x)) - 1; keep it gentle, so use x directly.
    let centered = 2.0 / (1.0 + (-x).exp()) - 1.0;
    centered.max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn day(n: i64) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 1 + n as u32, 0, 0, 0)
            .unwrap()
    }

    fn series(values: &[(i64, u64)]) -> Vec<(DateTime<Utc>, u64)> {
        values.iter().map(|(d, v)| (day(*d), *v)).collect()
    }

    #[test]
    fn no_data_is_zero_and_explainable() {
        let input = MomentumInput::default();
        let score = momentum_for_project(&ProjectId("tokio-rs/tokio".into()), &input);
        assert_eq!(score.total, 0.0);
        assert_eq!(score.stars, 0.0);
    }

    #[test]
    fn high_star_velocity_scores_high() {
        let input = MomentumInput {
            stars: series(&[(0, 1_000), (1, 1_200), (2, 1_400)]),
            ..Default::default()
        };
        let score = momentum_for_project(&ProjectId("foo/bar".into()), &input);
        assert!(score.stars > 0.9, "stars={}", score.stars);
        assert!(score.total > 0.0);
        assert!(score.total < 1.0);
    }

    #[test]
    fn slow_star_velocity_scores_low() {
        let input = MomentumInput {
            stars: series(&[(0, 1_000), (1, 1_001), (2, 1_002)]),
            ..Default::default()
        };
        let score = momentum_for_project(&ProjectId("foo/bar".into()), &input);
        assert!(score.stars < 0.1, "stars={}", score.stars);
    }

    #[test]
    fn weights_are_applied() {
        let input = MomentumInput {
            stars: series(&[(0, 1_000), (1, 1_200)]),
            downloads: series(&[(0, 10_000), (1, 12_000)]),
            ..Default::default()
        };
        let score = momentum_for_project(&ProjectId("foo/bar".into()), &input);
        let no_downloads = momentum_for_project(
            &ProjectId("foo/bar".into()),
            &MomentumInput {
                stars: series(&[(0, 1_000), (1, 1_200)]),
                ..Default::default()
            },
        );
        assert!(score.total > no_downloads.total);
    }

    #[test]
    fn flat_series_has_zero_momentum() {
        let input = MomentumInput {
            stars: series(&[(0, 1_000), (1, 1_000), (2, 1_000)]),
            ..Default::default()
        };
        let score = momentum_for_project(&ProjectId("foo/bar".into()), &input);
        assert_eq!(score.stars, 0.0);
    }

    #[test]
    fn total_is_exactly_weighted_sum() {
        let input = MomentumInput {
            stars: series(&[(0, 1_000), (1, 1_500), (2, 2_000)]),
            downloads: series(&[(0, 5_000), (1, 9_000), (2, 13_000)]),
            contributors: series(&[(0, 10), (1, 12), (2, 14)]),
            releases: series(&[(0, 1), (1, 3), (2, 5)]),
            community: series(&[(0, 2), (1, 5), (2, 8)]),
        };
        let score = momentum_for_project(&ProjectId("foo/bar".into()), &input);
        let expected = DEFAULT_WEIGHTS.stars * score.stars
            + DEFAULT_WEIGHTS.downloads * score.downloads
            + DEFAULT_WEIGHTS.contributors * score.contributors
            + DEFAULT_WEIGHTS.releases * score.releases
            + DEFAULT_WEIGHTS.community * score.community;
        assert!((score.total - expected).abs() < 1e-12);
    }
}
