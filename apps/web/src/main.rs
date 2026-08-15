//! SSR entry point: Axum server serving the Leptos app and its API.

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use tracing_subscriber::EnvFilter;

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Read the leptos config from this package's Cargo.toml so the binary
    // also runs without cargo-leptos setting environment variables.
    let conf = get_configuration(Some("Cargo.toml")).expect("leptos configuration is required");
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(ferris_web::app::App);

    // The dashboard stays functional (with honest "no data" states) when
    // PostgreSQL is unavailable; the API reports the database as offline.
    let pool: Option<ferris_database::DbPool> = match std::env::var("DATABASE_URL") {
        Ok(url) if !url.is_empty() => match ferris_database::connect_and_migrate(&url).await {
            Ok(pool) => {
                log!("connected to PostgreSQL");
                Some(pool)
            }
            Err(err) => {
                log!("warning: could not connect to PostgreSQL: {err}");
                None
            }
        },
        _ => {
            log!("warning: DATABASE_URL not set; API will report database offline");
            None
        }
    };

    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            move || provide_context(pool.clone()),
            {
                let leptos_options = leptos_options.clone();
                move || ferris_web::app::shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(ferris_web::app::shell))
        .with_state(leptos_options);

    log!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind site address");
    axum::serve(listener, app.into_make_service())
        .await
        .expect("server error");
}

#[cfg(not(feature = "ssr"))]
fn main() {
    eprintln!("ferris-web binary requires the `ssr` feature (cargo-leptos provides it)");
}
