//! Projects with explainable momentum scores.

use crate::api::get_trending;
use crate::components::MomentumBar;
use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn ProjectsPage() -> impl IntoView {
    let trending = Resource::new(|| (), |_| get_trending());

    view! {
        <Title text="Projects — Ferris Dashboard"/>
        <section class="page-head">
            <h1>"Projects"</h1>
            <p class="page-sub">
                "GitHub projects ranked by momentum. Scores are explainable: every component is shown."
            </p>
        </section>

        <Transition fallback=|| view! { <p class="muted">"Loading…"</p> }>
            {move || {
                trending.get().map(|result| match result {
                    Ok(list) if !list.is_empty() => view! {
                        <div class="project-table">
                            <div class="project-row project-row--head">
                                <span>"Project"</span>
                                <span>"Stars"</span>
                                <span>"Momentum breakdown"</span>
                            </div>
                            {list
                                .iter()
                                .map(|t| {
                                    view! {
                                        <div class="project-row">
                                            <div>
                                                <a class="project-name" href=t.project.html_url.clone() target="_blank" rel="noopener noreferrer">
                                                    {t.project.name.clone()}
                                                </a>
                                                {t.project.description.clone().map(|d| view! {
                                                    <div class="project-desc">{d}</div>
                                                })}
                                            </div>
                                            <span class="project-stars">
                                                {format_stars(t.project.stars)}
                                            </span>
                                            <MomentumBar score=t.score/>
                                        </div>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </div>
                    }.into_any(),
                    Ok(_) => view! { <p class="muted">"No project data yet — start the ingestion worker."</p> }.into_any(),
                    Err(_) => view! { <p class="muted">"Projects are unavailable (is PostgreSQL running?)."</p> }.into_any(),
                })
            }}
        </Transition>
    }
}

/// Compact star counts: 12345 → "12.3k".
fn format_stars(stars: u64) -> String {
    if stars >= 1_000_000 {
        format!("{:.1}M", stars as f64 / 1_000_000.0)
    } else if stars >= 1_000 {
        format!("{:.1}k", stars as f64 / 1_000.0)
    } else {
        stars.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::format_stars;

    #[test]
    fn formats_star_counts() {
        assert_eq!(format_stars(0), "0");
        assert_eq!(format_stars(999), "999");
        assert_eq!(format_stars(12_345), "12.3k");
        assert_eq!(format_stars(1_234_567), "1.2M");
    }
}
