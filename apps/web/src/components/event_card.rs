//! One normalized ecosystem event in a list.

use ferris_core::{EcosystemEvent, EventType};
use leptos::prelude::*;

/// CSS class for an event type badge.
pub fn event_type_class(event_type: EventType) -> &'static str {
    match event_type {
        EventType::Release => "badge badge--release",
        EventType::Security => "badge badge--security",
        EventType::Language => "badge badge--language",
        EventType::Compiler => "badge badge--compiler",
        EventType::Library => "badge badge--library",
        EventType::Framework => "badge badge--framework",
        EventType::Tooling => "badge badge--tooling",
        EventType::Project => "badge badge--project",
        EventType::Community => "badge badge--community",
        EventType::Conference => "badge badge--conference",
        EventType::Adoption => "badge badge--adoption",
        EventType::Performance => "badge badge--performance",
        EventType::Funding => "badge badge--funding",
        EventType::Job => "badge badge--job",
        EventType::Other => "badge badge--other",
    }
}

/// A single event row: type badge, title, source, relative time.
#[component]
pub fn EventCard(event: EcosystemEvent) -> impl IntoView {
    let badge = event_type_class(event.event_type);
    let time = relative_time(event.published_at);

    view! {
        <article class="event-card">
            <span class=badge>{event.event_type.label()}</span>
            <div class="event-body">
                <a class="event-title" href=event.url target="_blank" rel="noopener noreferrer">
                    {event.title}
                </a>
                <div class="event-meta">
                    <span class="event-source">{event.source.label()}</span>
                    <span class="event-time">{time}</span>
                </div>
            </div>
        </article>
    }
}

/// Compact relative timestamp like "2h ago" or "3d ago".
pub fn relative_time(published_at: Option<chrono::DateTime<chrono::Utc>>) -> String {
    let Some(published_at) = published_at else {
        return "unknown time".into();
    };
    let delta = chrono::Utc::now().signed_duration_since(published_at);
    if delta < chrono::Duration::minutes(1) {
        "just now".into()
    } else if delta < chrono::Duration::hours(1) {
        format!("{}m ago", delta.num_minutes())
    } else if delta < chrono::Duration::days(1) {
        format!("{}h ago", delta.num_hours())
    } else {
        format!("{}d ago", delta.num_days())
    }
}
