use std::{env, net::SocketAddr};

use anyhow::{Context, Result};
use axum::{middleware, routing::get, Json, Router};
use backend::{admin, auth, db, state::AppState};
use serde::Serialize;
use tracing::info;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    database_url_configured: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let db_settings = db::DatabaseSettings::from_env()?;
    let pool = db::connect(&db_settings).await?;
    db::run_migrations(&pool).await?;
    let jwt_config = auth::jwt::JwtConfig::from_env()?;

    let port = read_port("BACKEND_PORT")?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let state = AppState {
        db_pool: pool,
        jwt_config,
    };
    let admin_routes = admin::routes::router().route_layer(middleware::from_fn_with_state(
        state.clone(),
        auth::middleware::require_admin_auth,
    ));

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .merge(auth::routes::router())
        .merge(admin_routes)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind backend listener on {addr}"))?;

    info!("backend listening on http://{addr}");
    axum::serve(listener, app)
        .await
        .context("backend server exited with error")?;

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

async fn root_handler() -> &'static str {
    "blog002 backend"
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        database_url_configured: env::var("DATABASE_URL").is_ok(),
    })
}
