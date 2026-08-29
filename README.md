# Ferris Dashboard

```
███████╗███████╗██████╗ ██████╗ ██╗ ██████╗
██╔════╝██╔════╝██╔══██╗██╔══██╗██║██╔════╝
█████╗  █████╗  ██████╔╝██████╔╝██║███████╗
██╔══╝  ██╔══╝  ██╔══██╗██╔══██╗██║╚════██║
██║     ███████╗██║  ██║██║  ██║██║██████╔╝
╚═╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝╚═════╝
```

---

## ◆ PULSE

*What matters in Rust right now?* The ecosystem publishes in five
places - the Rust Blog, This Week in Rust, crates.io, GitHub,
RustSec - and no one reads all five daily. Ferris Dashboard is the
observability layer that does: ingestion workers pull each source,
connectors normalize it into one domain model, a momentum engine
scores the signals with explainable breakdowns, and a Leptos SSR
dashboard renders what matters. The ecosystem's pulse, taken
automatically.

| Ingest ▣ | Normalize ▣ | Momentum ▣ | SSR dashboard ▣ |
|---|---|---|---|

*The pipeline - ingest, dedup, score, render - is being forged.*

> Built with Rust, Leptos SSR + Axum, SQLx on PostgreSQL - dark
> engineering surfaces, red accent, information density over
> decoration.
>
> **suradet-ps**, artifact keeper

---

## ◆ IGNITION

Four steps, two processes.

```
⟫ cp .env.example .env
⟫ createdb ferris_dashboard
⟫ sqlx migrate run
⟫ cargo run -p ferris-worker    # ingestion worker
⟫ cargo run -p ferris-web --features ssr
```

The dashboard answers at [http://127.0.0.1:3000](http://127.0.0.1:3000).
For hydration and hot reload: `⟫ cargo leptos watch`.

<details>
<summary>Prerequisites</summary>

- Rust stable (see `rust-toolchain.toml`)
- PostgreSQL 14+
- [cargo-leptos](https://github.com/leptos-rs/cargo-leptos) for the
  web app's WASM hydration bundle

</details>

---

## ◆ ANATOMY

One pipeline, dependencies flowing inward, each crate knowing its
place.

- **Ingests** - `apps/worker` runs the scheduler with source
  isolation and retries; each `crates/source-*` connector does one
  job: fetch, parse, normalize.
- **Normalizes** - `crates/ingest` orchestrates the connectors,
  normalizes into the common domain model, and deduplicates - five
  sources become one vocabulary.
- **Models** - `crates/core` holds the domain: provenance, event
  classification, and the facts every source is reduced to.
- **Scores** - `crates/scoring` is the momentum engine with
  explainable score breakdowns - a number that can say why.
- **Stores** - `crates/database` persists through SQLx repositories
  into versioned PostgreSQL migrations - the record survives the
  processes.
- **Renders** - `apps/web` serves the Leptos SSR dashboard with
  hydration, the logo, and the dark theme - density over decoration,
  red `#8f1f1d` over engineering dark.

---

## ◆ RITUALS

**The core ceremony** - the ecosystem scan:

1. Start the worker; the sources are fetched on their schedules, each
   isolated, each retried on failure.
2. Watch the connectors normalize and deduplicate into one domain
   model - five streams, one vocabulary.
3. Open the dashboard. The momentum engine's scores answer, with
   breakdowns that explain themselves.
4. Read the pulse: what the ecosystem did while the reader was
   elsewhere.

**The ceremony of the inward flow** - dependencies point inward:
connectors to ingest to core, scoring and database beside it, the
application on top. Each crate imports what it may, and the graph
stays honest.

**The ceremony of the explainable score** - momentum without a
breakdown is a mood. Every number carries its reasons, so the reader
can agree or disagree - with evidence.

---

## ◆ ECHOES

**Where this artifact is heading**

```
ingest ▸ source connectors, scheduler, retries ─────────────────────── ▸ forging
model  ▸ common domain, provenance, classification ─────────────────── ▸ forging
score  ▸ momentum engine, explainable breakdowns ───────────────────── ▸ forging
serve  ▸ Leptos SSR dashboard, dark engineering theme ──────────────── ▸ forging
```

**Raising the artifact** - the sources, the schema, and the scoring
live in `crates/` and `migrations/`; the dashboard in `apps/web/`.
Open an issue first to discuss a change.

**Status** - dependencies are maintained through Renovate; the
workspace is being forged from the scaffold up.

---

```
  ─────────────────────────────────────────
   An ecosystem is read daily by no one.
   A dashboard is read daily by everyone.
  ─────────────────────────────────────────
```

Open source.