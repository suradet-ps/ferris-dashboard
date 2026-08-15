//! Explainable momentum bar (AGENTS.md §18): every component is visible,
//! so the UI can answer "why is this project trending?".

use ferris_core::ScoreBreakdown;
use leptos::prelude::*;

/// (component name, CSS class) in display order.
const SEGMENTS: [(&str, &str); 5] = [
    ("stars", "seg--stars"),
    ("downloads", "seg--downloads"),
    ("contributors", "seg--contributors"),
    ("releases", "seg--releases"),
    ("community", "seg--community"),
];

#[component]
pub fn MomentumBar(score: ScoreBreakdown) -> impl IntoView {
    let values = [
        score.stars,
        score.downloads,
        score.contributors,
        score.releases,
        score.community,
    ];

    view! {
        <div class="momentum">
            <div class="momentum-bar"
                role="img"
                aria-label=format!(
                    "momentum {:.0}% — stars {:.0}%, downloads {:.0}%, contributors {:.0}%, releases {:.0}%, community {:.0}%",
                    score.total * 100.0,
                    score.stars * 100.0,
                    score.downloads * 100.0,
                    score.contributors * 100.0,
                    score.releases * 100.0,
                    score.community * 100.0,
                )
            >
                {values
                    .into_iter()
                    .enumerate()
                    .map(|(i, value)| {
                        let pct = (value.clamp(0.0, 1.0) * 100.0) as u32;
                        let class = SEGMENTS[i].1;
                        view! { <span class=class style=format!("width: {pct}%")></span> }
                    })
                    .collect::<Vec<_>>()}
            </div>
            <div class="momentum-total">
                "momentum " <strong>{format!("{:.0}%", score.total * 100.0)}</strong>
            </div>
        </div>
    }
}
