use std::{env, net::SocketAddr};

use anyhow::{Context, Result};
use axum::{http::header, response::Html, routing::get, Router};
use leptos::prelude::*;
use tracing::info;

mod components;
use components::layout::BaseLayout;

#[component]
fn App() -> impl IntoView {
    view! {
        <BaseLayout>
            <div class="grid gap-6 md:grid-cols-[1.4fr_1fr] md:items-center">
                <div class="space-y-4">
                    <p class="inline-flex rounded-full border border-cyan-400/40 px-3 py-1 text-xs tracking-[0.18em] text-cyan-200">
                        "TAILWIND + LEPTOS"
                    </p>
                    <h1 class="text-3xl font-semibold tracking-tight text-white sm:text-4xl">
                        "Frontend foundation is ready"
                    </h1>
                    <p class="max-w-xl text-sm leading-relaxed text-slate-300 sm:text-base">
                        "Tailwind CSS is wired into the frontend pipeline with a reusable base layout."
                    </p>
                </div>

                <aside class="rounded-xl border border-cyan-500/20 bg-cyan-500/10 p-5 text-sm text-cyan-100">
                    <p class="font-medium uppercase tracking-wider text-cyan-200">"Next UI Steps"</p>
                    <ul class="mt-3 space-y-2 text-cyan-50/90">
                        <li>"Admin login page"</li>
                        <li>"Category and tag management"</li>
                        <li>"Post create/edit screens"</li>
                    </ul>
                </aside>
            </div>
        </BaseLayout>
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let port = read_port("FRONTEND_PORT")?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/styles.css", get(styles_handler));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind frontend listener on {addr}"))?;

    info!("frontend listening on http://{addr}");
    axum::serve(listener, app)
        .await
        .context("frontend server exited with error")?;

    Ok(())
}

fn init_tracing() {
    let filter = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

fn read_port(var_name: &str) -> Result<u16> {
    let fallback = env::var("PORT").ok();
    let raw = env::var(var_name)
        .ok()
        .or(fallback)
        .unwrap_or_else(|| "8080".to_string());

    raw.parse::<u16>()
        .with_context(|| format!("invalid port value for {var_name}/PORT: {raw}"))
}

async fn index_handler() -> Html<String> {
    let app_html = leptos::ssr::render_to_string(|| view! { <App /> });
    let document = format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>blog002</title><link rel=\"stylesheet\" href=\"/styles.css\"></head><body>{app_html}</body></html>"
    );

    Html(document)
}

async fn styles_handler() -> ([(&'static str, &'static str); 1], &'static str) {
    (
        [(header::CONTENT_TYPE.as_str(), "text/css; charset=utf-8")],
        include_str!("../styles/tailwind.css"),
    )
}
