use std::{env, net::SocketAddr};

use anyhow::{Context, Result};
use axum::{response::Html, routing::get, Router};
use leptos::prelude::*;
use tracing::info;

#[component]
fn App() -> impl IntoView {
    view! {
        <main>
            <h1>"blog002 frontend"</h1>
            <p>"Leptos app scaffold initialized."</p>
        </main>
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let port = read_port("FRONTEND_PORT")?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let app = Router::new().route("/", get(index_handler));

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
    let html = leptos::ssr::render_to_string(|| view! { <App /> });
    Html(html)
}
