//! Latest events with deterministic type filtering.

use crate::api::get_events;
use crate::components::EventCard;
use leptos::prelude::*;
use leptos_meta::Title;

/// Filter options shown as buttons (deterministic classification, no
/// free-form tags — AGENTS.md §16).
const FILTERS: [(&str, Option<&str>); 8] = [
    ("All", None),
    ("Releases", Some("release")),
    ("Security", Some("security")),
    ("Language", Some("language")),
    ("Compiler", Some("compiler")),
    ("Tooling", Some("tooling")),
    ("Framework", Some("framework")),
    ("Community", Some("community")),
];

#[component]
pub fn EventsPage() -> impl IntoView {
    let filter = RwSignal::new(None::<String>);

    let events = Resource::new(move || filter.get(), get_events);

    view! {
        <Title text="Events — Ferris Dashboard"/>
        <section class="page-head">
            <h1>"Events"</h1>
            <p class="page-sub">"Normalized ecosystem events, newest first, one report per event."</p>
        </section>

        <div class="filter-bar" role="group" aria-label="Filter events by type">
            {FILTERS
                .into_iter()
                .map(|(label, value)| {
                    let active = move || filter.get().as_deref() == value;
                    let set_value = value.map(str::to_string);
                    view! {
                        <button
                            class:active=active
                            on:click=move |_| filter.set(set_value.clone())
                        >
                            {label}
                        </button>
                    }
                })
                .collect::<Vec<_>>()}
        </div>

        <Transition fallback=|| view! { <p class="muted">"Loading…"</p> }>
            {move || {
                events.get().map(|result| match result {
                    Ok(list) if !list.is_empty() => view! {
                        <div class="event-list">
                            {list
                                .iter()
                                .cloned()
                                .map(|e| view! { <EventCard event=e/> })
                                .collect::<Vec<_>>()}
                        </div>
                    }.into_any(),
                    Ok(_) => view! { <p class="muted">"No events in this category yet."</p> }.into_any(),
                    Err(_) => view! { <p class="muted">"Events are unavailable (is PostgreSQL running?)."</p> }.into_any(),
                })
            }}
        </Transition>
    }
}
