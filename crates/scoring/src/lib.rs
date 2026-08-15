//! The momentum engine.
//!
//! Momentum MUST NOT be based on a single metric (AGENTS.md §17). The exact
//! formula lives here and nowhere else — not in Leptos components, not in
//! SQL queries, not in HTTP handlers, not in source connectors.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod momentum;
pub mod velocity;

pub use momentum::{MomentumInput, MomentumWeights, momentum_for_crate, momentum_for_project};
pub use velocity::linear_velocity;
