//! Time-series velocity estimation.
//!
//! Velocity is the slope of a linear regression over the rolling window.
//! This is deliberately simple, deterministic and testable; anomaly
//! detection can be layered on top later.

use chrono::{DateTime, Utc};

/// Estimate the per-day slope of `points` (least squares).
///
/// Returns `0.0` when fewer than two points are available.
///
/// # Formula
///
/// ```text
/// slope = Σ((t_i - t̄)(v_i - v̄)) / Σ((t_i - t̄)²)
/// ```
/// where `t` is measured in days since the first observation.
pub fn linear_velocity(points: &[(DateTime<Utc>, f64)]) -> f64 {
    if points.len() < 2 {
        return 0.0;
    }
    let t0 = points[0].0;
    let t: Vec<f64> = points
        .iter()
        .map(|(ts, _)| ts.signed_duration_since(t0).num_seconds() as f64 / 86_400.0)
        .collect();
    let v: Vec<f64> = points.iter().map(|(_, value)| *value).collect();
    let n = t.len() as f64;

    let t_mean = t.iter().sum::<f64>() / n;
    let v_mean = v.iter().sum::<f64>() / n;

    let mut numerator = 0.0;
    let mut denominator = 0.0;
    for (ti, vi) in t.iter().zip(v.iter()) {
        numerator += (ti - t_mean) * (vi - v_mean);
        denominator += (ti - t_mean) * (ti - t_mean);
    }
    if denominator == 0.0 {
        return 0.0;
    }
    numerator / denominator
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn day(n: i64) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 1 + n as u32, 0, 0, 0)
            .unwrap()
    }

    #[test]
    fn constant_series_has_zero_velocity() {
        let points = vec![(day(0), 100.0), (day(1), 100.0), (day(2), 100.0)];
        assert_eq!(linear_velocity(&points), 0.0);
    }

    #[test]
    fn rising_series_has_positive_velocity() {
        let points = vec![(day(0), 100.0), (day(1), 120.0), (day(2), 140.0)];
        let v = linear_velocity(&points);
        assert!((v - 20.0).abs() < 1e-6, "expected 20/day, got {v}");
    }

    #[test]
    fn single_point_has_zero_velocity() {
        assert_eq!(linear_velocity(&[(day(0), 42.0)]), 0.0);
    }

    #[test]
    fn empty_series_has_zero_velocity() {
        assert_eq!(linear_velocity(&[]), 0.0);
    }
}
