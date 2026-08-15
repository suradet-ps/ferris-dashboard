//! Momentum engine behavior across components.

use chrono::{TimeZone, Utc};
use ferris_core::{CrateId, ProjectId};
use ferris_scoring::{MomentumInput, momentum_for_crate, momentum_for_project};

fn day(n: i64) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 1 + n as u32, 0, 0, 0).unwrap()
}

fn series(values: &[(i64, u64)]) -> Vec<(chrono::DateTime<Utc>, u64)> {
    values.iter().map(|(d, v)| (day(*d), *v)).collect()
}

#[test]
fn project_with_strong_signals_outscores_quiet_project() {
    let loud = MomentumInput {
        stars: series(&[(0, 500), (1, 900), (2, 1400), (3, 2000)]),
        downloads: series(&[(0, 10_000), (1, 40_000), (2, 90_000)]),
        contributors: series(&[(0, 5), (1, 12), (2, 25)]),
        releases: series(&[(0, 1), (1, 2), (2, 5)]),
        community: series(&[(0, 1), (1, 3), (2, 9)]),
    };
    let quiet = MomentumInput {
        stars: series(&[(0, 500), (1, 505), (2, 510)]),
        ..Default::default()
    };

    let loud_score = momentum_for_project(&ProjectId("a/b".into()), &loud);
    let quiet_score = momentum_for_project(&ProjectId("a/b".into()), &quiet);

    assert!(loud_score.total > quiet_score.total);
    assert!(loud_score.stars > 0.9);
    assert_eq!(quiet_score.total, quiet_score.stars * 0.25);
}

#[test]
fn crate_score_favors_downloads() {
    let growing = MomentumInput {
        downloads: series(&[(0, 1_000), (1, 30_000), (2, 90_000)]),
        ..Default::default()
    };
    let score = momentum_for_crate(&CrateId("serde".into()), &growing);
    // Downloads dominate the crate formula (weight 0.5).
    assert!(score.downloads > 0.9);
    assert!(score.total > 0.45);
}

#[test]
fn scores_are_bounded_and_explainable() {
    let input = MomentumInput {
        stars: series(&[(0, 100), (1, 100_000), (2, 500_000)]),
        downloads: series(&[(0, 0), (1, 10_000_000), (2, 50_000_000)]),
        ..Default::default()
    };
    let score = momentum_for_project(&ProjectId("x/y".into()), &input);
    assert!((0.0..=1.0).contains(&score.total));
    for component in [score.stars, score.downloads] {
        assert!((0.0..=1.0).contains(&component));
    }
}
