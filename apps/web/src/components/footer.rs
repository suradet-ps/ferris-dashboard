//! Site footer.

use leptos::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="footer">
            <div class="container">
                <p>
                    "Ferris Dashboard — what matters in Rust right now. "
                    "Data is aggregated server-side; source freshness is shown in the header."
                </p>
            </div>
        </footer>
    }
}
