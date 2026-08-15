//! Overview stat card.

use leptos::prelude::*;

#[component]
pub fn StatCard(
    #[prop(into)] label: String,
    #[prop(into)] value: String,
    #[prop(optional, into)] hint: String,
    #[prop(optional)] accent: bool,
) -> impl IntoView {
    let class = if accent { "stat stat--accent" } else { "stat" };
    view! {
        <div class=class>
            <div class="stat-value">{value}</div>
            <div class="stat-label">{label}</div>
            {if hint.is_empty() {
                None
            } else {
                Some(view! { <div class="stat-hint">{hint}</div> }.into_any())
            }}
        </div>
    }
}
