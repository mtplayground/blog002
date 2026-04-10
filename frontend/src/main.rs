use std::{env, net::SocketAddr};

use anyhow::{Context, Result};
use axum::{http::header, response::Html, routing::get, Router};
use leptos::prelude::*;
use tracing::info;

mod api;
mod components;
mod pages;
mod state;

use components::{image_upload::ImageUpload, layout::BaseLayout};
use pages::{
    admin_categories::AdminCategoriesPage, admin_login::AdminLoginPage, admin_tags::AdminTagsPage,
};
use state::auth::use_or_provide_auth_context;

#[component]
fn HomePage() -> impl IntoView {
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
                        "Admin login is available at /admin/login with JWT persistence in context and localStorage."
                    </p>
                </div>

                <aside class="rounded-xl border border-cyan-500/20 bg-cyan-500/10 p-5 text-sm text-cyan-100">
                    <p class="font-medium uppercase tracking-wider text-cyan-200">"Quick Links"</p>
                    <ul class="mt-3 space-y-2 text-cyan-50/90">
                        <li><a class="transition hover:text-white" href="/admin/login">"Admin login"</a></li>
                        <li><a class="transition hover:text-white" href="/admin">"Admin dashboard"</a></li>
                    </ul>
                </aside>
            </div>
        </BaseLayout>
    }
}

#[component]
fn AdminDashboardPage() -> impl IntoView {
    let auth = use_or_provide_auth_context();
    let uploaded_image_url = RwSignal::new(None::<String>);

    view! {
        <BaseLayout>
            <div class="space-y-4">
                <h1 class="text-2xl font-semibold text-white">"Admin dashboard"</h1>
                <p class="text-sm text-slate-300">
                    "This page is the post-login redirect target. Protected API calls can use the token stored in auth context."
                </p>
                <p class="rounded-lg border border-slate-700 bg-slate-900/60 px-3 py-2 text-xs text-slate-300">
                    {move || match auth.token() {
                        Some(_) => "JWT is loaded in memory/localStorage.".to_string(),
                        None => "No JWT found. Sign in at /admin/login.".to_string(),
                    }}
                </p>
                <div class="flex flex-wrap gap-2">
                    <a
                        class="inline-flex items-center rounded-lg border border-slate-700 px-3 py-2 text-sm text-slate-200 transition hover:border-slate-500 hover:text-white"
                        href="/admin/categories"
                    >
                        "Manage categories"
                    </a>
                    <a
                        class="inline-flex items-center rounded-lg border border-slate-700 px-3 py-2 text-sm text-slate-200 transition hover:border-slate-500 hover:text-white"
                        href="/admin/tags"
                    >
                        "Manage tags"
                    </a>
                </div>

                <ImageUpload
                    token=Signal::derive(move || auth.token())
                    on_uploaded=Callback::new(move |url: String| {
                        uploaded_image_url.set(Some(url));
                    })
                />

                <Show
                    when=move || uploaded_image_url.get().is_some()
                    fallback=|| view! { <></> }
                >
                    <p class="rounded-lg border border-cyan-500/30 bg-cyan-500/10 px-3 py-2 text-xs text-cyan-100">
                        {move || format!(
                            "Latest uploaded URL: {}",
                            uploaded_image_url.get().unwrap_or_default()
                        )}
                    </p>
                </Show>
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
        .route("/", get(home_handler))
        .route("/admin", get(admin_dashboard_handler))
        .route("/admin/login", get(admin_login_handler))
        .route("/admin/categories", get(admin_categories_handler))
        .route("/admin/tags", get(admin_tags_handler))
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

async fn home_handler() -> Html<String> {
    render_document(leptos::ssr::render_to_string(|| view! { <HomePage /> }))
}

async fn admin_login_handler() -> Html<String> {
    render_document(leptos::ssr::render_to_string(|| view! { <AdminLoginPage /> }))
}

async fn admin_dashboard_handler() -> Html<String> {
    render_document(leptos::ssr::render_to_string(|| view! { <AdminDashboardPage /> }))
}

async fn admin_categories_handler() -> Html<String> {
    render_document(leptos::ssr::render_to_string(|| view! { <AdminCategoriesPage /> }))
}

async fn admin_tags_handler() -> Html<String> {
    render_document(leptos::ssr::render_to_string(|| view! { <AdminTagsPage /> }))
}

fn render_document(app_html: String) -> Html<String> {
    Html(format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>blog002</title><link rel=\"stylesheet\" href=\"/styles.css\"></head><body>{app_html}</body></html>"
    ))
}

async fn styles_handler() -> ([(&'static str, &'static str); 1], &'static str) {
    (
        [(header::CONTENT_TYPE.as_str(), "text/css; charset=utf-8")],
        include_str!("../styles/tailwind.css"),
    )
}
