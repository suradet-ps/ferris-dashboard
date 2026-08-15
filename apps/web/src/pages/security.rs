//! Security advisories (RustSec).

use crate::api::get_security;
use crate::components::relative_time;
use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn SecurityPage() -> impl IntoView {
    let advisories = Resource::new(|| (), |_| get_security());

    view! {
        <Title text="Security — Ferris Dashboard"/>
        <section class="page-head">
            <h1>"Security"</h1>
            <p class="page-sub">"RustSec advisories affecting crates.io."</p>
        </section>

        <Transition fallback=|| view! { <p class="muted">"Loading…"</p> }>
            {move || {
                advisories.get().map(|result| match result {
                    Ok(list) if !list.is_empty() => view! {
                        <div class="event-list">
                            {list
                                .iter()
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
                                                {a.summary.clone().map(|s| view! {
                                                    <p class="event-summary">{s}</p>
                                                })}
                                            </div>
                                        </article>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </div>
                    }.into_any(),
                    Ok(_) => view! { <p class="muted">"No advisories yet."</p> }.into_any(),
                    Err(_) => view! { <p class="muted">"Advisories are unavailable (is PostgreSQL running?)."</p> }.into_any(),
                })
            }}
        </Transition>
    }
}
