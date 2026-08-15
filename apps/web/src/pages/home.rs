//! Overview: "What is happening in Rust right now?" (AGENTS.md §26).

use crate::api::{DashboardData, get_dashboard};
use crate::components::{EventCard, MomentumBar, StatCard, relative_time};
use ferris_core::SourceStatus;
use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn HomePage() -> impl IntoView {
    let data = Resource::new(|| (), |_| get_dashboard());

    view! {
        <Title text="Ferris Dashboard — Rust Ecosystem Observability"/>
        <section class="page-head">
            <h1>"Rust ecosystem, right now"</h1>
            <p class="page-sub">"Ingested from Rust Blog, TWIR, crates.io, GitHub, RustSec and RFCs."</p>
        </section>

        <Transition fallback=|| view! { <p class="muted">"Loading…"</p> }>
            {move || {
                data.get().map(|result| match result {
                    Ok(d) => view! { <DashboardBody data=d/> }.into_any(),
                    Err(_) => view! { <DatabaseOffline/> }.into_any(),
                })
            }}
        </Transition>
    }
}

#[component]
fn DashboardBody(data: DashboardData) -> impl IntoView {
    let stats = data.stats;
    view! {
        <section class="stats-grid" aria-label="Overview">
            <StatCard label="Events (24h)" value=stats.events_24h.to_string() hint=format!("{} total", stats.events_total) accent=true/>
            <StatCard label="Projects tracked" value=stats.projects.to_string()/>
            <StatCard label="Crates tracked" value=stats.crates.to_string()/>
            <StatCard label="Security advisories" value=stats.advisories.to_string()/>
            <StatCard label="Sources healthy" value=format!("{}/{}", stats.healthy_sources, stats.healthy_sources + stats.degraded_sources)/>
        </section>

        <section class="grid-2">
            <section class="panel">
                <h2 class="panel-title">"Latest events"</h2>
                {if data.latest_events.is_empty() {
                    view! { <p class="muted">"No events yet — start the ingestion worker."</p> }.into_any()
                } else {
                    view! {
                        <div class="event-list">
                            {data.latest_events
                                .iter()
                                .take(10)
                                .cloned()
                                .map(|e| view! { <EventCard event=e/> })
                                .collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                }}
            </section>

            <section class="panel">
                <h2 class="panel-title">"Trending"</h2>
                {if data.trending.is_empty() {
                    view! { <p class="muted">"No trending data yet."</p> }.into_any()
                } else {
                    view! {
                        <div class="trend-list">
                            {data.trending
                                .iter()
                                .take(8)
                                .map(|t| {
                                    view! {
                                        <div class="trend-row">
                                            <a class="trend-name" href=t.project.html_url.clone() target="_blank" rel="noopener noreferrer">
                                                {t.project.name.clone()}
                                            </a>
                                            <MomentumBar score=t.score/>
                                        </div>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                }}
            </section>
        </section>

        <section class="grid-2">
            <section class="panel">
                <h2 class="panel-title">"Top crates (recent downloads)"</h2>
                {if data.top_crates.is_empty() {
                    view! { <p class="muted">"No crate data yet."</p> }.into_any()
                } else {
                    view! {
                        <ol class="rank-list">
                            {data.top_crates
                                .iter()
                                .take(10)
                                .map(|c| {
                                    view! {
                                        <li>
                                            <a href=format!("https://crates.io/crates/{}", c.name) target="_blank" rel="noopener noreferrer">
                                                {c.name.clone()}
                                            </a>
                                            <span class="rank-meta">
                                                {format!("{} | {} downloads (90d)", c.max_version, c.recent_downloads)}
                                            </span>
                                        </li>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </ol>
                    }.into_any()
                }}
            </section>

            <section class="panel">
                <h2 class="panel-title">"Recent advisories"</h2>
                {if data.recent_advisories.is_empty() {
                    view! { <p class="muted">"No advisories yet."</p> }.into_any()
                } else {
                    view! {
                        <div class="event-list">
                            {data.recent_advisories
                                .iter()
                                .take(6)
                                .map(|a| {
                                    let time = relative_time(a.published_at);
                                    view! {
                                        <article class="event-card">
                                            <span class="badge badge--security">"advisory"</span>
                                            <div class="event-body">
                                                <a class="event-title" href=a.url.clone() target="_blank" rel="noopener noreferrer">
                                                    {a.title.clone()}
                                                </a>
                                                <div class="event-meta">
                                                    <span class="event-source">{a.id.clone()}</span>
                                                    <span class="event-time">{time}</span>
                                                </div>
                                            </div>
                                        </article>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                }}
            </section>
        </section>

        <section class="panel">
            <h2 class="panel-title">"Source health"</h2>
            <div class="health-grid">
                {data.health
                    .iter()
                    .map(|h| {
                        let status_class = match h.status {
                            SourceStatus::Healthy => "status status--ok",
                            SourceStatus::Degraded => "status status--warn",
                            SourceStatus::Disabled => "status status--off",
                        };
                        let last = h
                            .last_fetched_at
                            .map(|t| relative_time(Some(t)))
                            .unwrap_or_else(|| "never".into());
                        view! {
                            <div class="health-cell">
                                <div class="health-name">{h.source.label()}</div>
                                <div>
                                    <span class=status_class>{h.status.label()}</span>
                                    <span class="health-meta">"updated " {last} " · every " {h.interval_minutes} "m"</span>
                                </div>
                            </div>
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>
        </section>
    }
}

/// Honest offline state: the app still renders when PostgreSQL is down.
#[component]
fn DatabaseOffline() -> impl IntoView {
    view! {
        <section class="panel">
            <h2 class="panel-title">"Database unavailable"</h2>
            <p class="muted">
                "The API is reachable, but PostgreSQL is not. Set "
                <code>"DATABASE_URL"</code>
                " and start the ingestion worker to populate the dashboard."
            </p>
        </section>
    }
}
