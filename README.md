# Ferris Dashboard

> **What matters in Rust right now?**

An observability dashboard for the Rust ecosystem. It ingests structured
information from multiple sources (Rust Blog, This Week in Rust, crates.io,
GitHub, RustSec), normalizes it into a common domain model, detects ecosystem
signals, calculates momentum, and presents the result through a fast Leptos
SSR dashboard.

## Architecture

```text
External Sources
       │
       ▼
 Ingestion Workers (apps/worker)
       │
       ▼
 Source Connectors (crates/source-*)
       │
       ▼
 Normalization + Dedup (crates/ingest)
       │
       ▼
 PostgreSQL (migrations/)
       │
       ▼
 Application API (apps/web, Leptos SSR + Axum)
       │
       ▼
 Dashboard
```

Dependencies flow inward: source connectors → ingest → core; scoring and
database sit beside core; the application layer consumes everything.

## Layout

```text
apps/web/          Leptos SSR + Axum dashboard (hydration, logo, dark theme)
apps/worker/       Ingestion worker (scheduler, source isolation, retries)
crates/core/       Domain models, provenance, event classification
crates/scoring/    Momentum engine with explainable score breakdowns
crates/database/   SQLx repositories (PostgreSQL)
crates/ingest/     Connector orchestration, normalization, deduplication
crates/source-*/   Isolated source connectors (fetch → parse → normalize)
migrations/        Versioned PostgreSQL migrations
```

## Prerequisites

- Rust stable (see `rust-toolchain.toml`)
- PostgreSQL 14+
- [cargo-leptos](https://github.com/leptos-rs/cargo-leptos) for the web app

## Setup

```text
cp .env.example .env
createdb ferris_dashboard
sqlx migrate run
cargo run -p ferris-worker     # ingestion worker
cargo run -p ferris-web --features ssr   # dashboard at http://127.0.0.1:3000
# or, for hydration + hot reload:
cargo leptos watch
```

The web binary reads its Leptos configuration from `apps/web/Cargo.toml`
(`cargo run` sets the working directory to the package, so this works
without cargo-leptos; WASM hydration still needs cargo-leptos to build the
`pkg/` bundle).

## Design language

The UI follows the logo: dark engineering surfaces, red accent `#8f1f1d`,
information density over decoration. See `apps/web/style/main.css`.
