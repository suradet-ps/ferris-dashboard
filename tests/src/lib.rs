//! Workspace-level integration tests.
//!
//! These tests run without a live network or database: every connector is
//! exercised against fixtures (AGENTS.md §34, §35).

pub mod ingestion;
pub mod momentum;
