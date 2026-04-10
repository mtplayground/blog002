use axum::{extract::Extension, routing::get, Json, Router};
use serde::Serialize;

use crate::auth::middleware::AuthenticatedAdmin;

#[derive(Debug, Serialize)]
struct AdminPingResponse {
    status: &'static str,
    admin_id: String,
    email: String,
    token_jti: String,
}

pub fn router() -> Router {
    Router::new().route("/api/admin/ping", get(admin_ping))
}

async fn admin_ping(Extension(admin): Extension<AuthenticatedAdmin>) -> Json<AdminPingResponse> {
    Json(AdminPingResponse {
        status: "ok",
        admin_id: admin.admin_id.to_string(),
        email: admin.email,
        token_jti: admin.jti,
    })
}
