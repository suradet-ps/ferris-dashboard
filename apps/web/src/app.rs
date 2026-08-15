//! Application shell: HTML document, router, layout.

use crate::components::footer::Footer;
use crate::components::header::Header;
use crate::pages::{
    events::EventsPage, home::HomePage, projects::ProjectsPage, security::SecurityPage,
};
use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, provide_meta_context};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

/// Renders the full HTML document for SSR.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en" data-theme="dark">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link rel="icon" type="image/svg+xml" href="/logo.svg"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

/// The application router and layout.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/ferris-web.css"/>
        <Router>
            <Header/>
            <main class="container">
                <Routes fallback=|| view! { <NotFound/> }>
                    <Route path=path!("/") view=HomePage/>
                    <Route path=path!("/events") view=EventsPage/>
                    <Route path=path!("/projects") view=ProjectsPage/>
                    <Route path=path!("/security") view=SecurityPage/>
                </Routes>
            </main>
            <Footer/>
        </Router>
    }
}

/// 404 fallback page.
#[component]
fn NotFound() -> impl IntoView {
    view! {
        <section class="empty-state">
            <h1>"404"</h1>
            <p>"That page is not part of the ecosystem."</p>
        </section>
    }
}
