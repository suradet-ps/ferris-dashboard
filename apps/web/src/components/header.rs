//! Site header: logo, wordmark, navigation, and live source freshness.

use crate::api::get_source_health;
use ferris_core::SourceStatus;
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Header() -> impl IntoView {
    let health = Resource::new(|| (), |_| get_source_health());

    view! {
        <header class="header">
            <div class="container header-inner">
                <div class="brand">
                    <A href="/" {..} class="brand-link">
                        <img src="/logo.svg" alt="Ferris Dashboard logo" class="brand-logo"/>
                        <span class="brand-name">"Ferris " <em>"Dashboard"</em></span>
                    </A>
                </div>

                <nav class="nav" aria-label="Primary">
                    <A href="/" exact=true>"Overview"</A>
                    <A href="/events">"Events"</A>
                    <A href="/projects">"Projects"</A>
                    <A href="/security">"Security"</A>
                </nav>

                <SourceFreshness health/>
            </div>
        </header>
    }
}

/// Compact source-health summary shown in the header (AGENTS.md §28).
#[component]
pub fn SourceFreshness(
    health: Resource<Result<Vec<ferris_core::SourceHealth>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <div class="freshness" title="Source health: healthy / degraded / disabled">
            <Transition fallback=|| view! { <span class="freshness-text">"sources …"</span> }>
                {move || {
                    health.get().map(|result| match result {
                        Ok(list) => {
                            let healthy =
                                list.iter().filter(|h| h.status == SourceStatus::Healthy).count();
                            let degraded = list
                                .iter()
                                .filter(|h| h.status == SourceStatus::Degraded)
                                .count();
                            view! {
                                <span class="freshness-dot dot-healthy"></span>
                                <span class="freshness-text">{healthy} ok</span>
                                <span class="freshness-dot dot-degraded"></span>
                                <span class="freshness-text">{degraded} degraded</span>
                            }
                                .into_any()
                        }
                        Err(_) => view! { <span class="freshness-text">"sources unavailable"</span> }
                            .into_any(),
                    })
                }}
            </Transition>
        </div>
    }
}
