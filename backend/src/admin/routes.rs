use axum::{
    extract::Extension,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;

use crate::{admin::categories::handlers as category_handlers, auth::middleware::AuthenticatedAdmin};

#[derive(Debug, Serialize)]
struct AdminPingResponse {
    status: &'static str,
    admin_id: String,
    email: String,
    token_jti: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/api/admin/ping", get(admin_ping))
        .route(
            "/api/admin/categories",
            post(category_handlers::create_category).get(category_handlers::list_categories),
        )
        .route(
            "/api/admin/categories/:id",
            get(category_handlers::get_category)
                .put(category_handlers::update_category)
                .delete(category_handlers::delete_category),
        )
}

async fn admin_ping(Extension(admin): Extension<AuthenticatedAdmin>) -> Json<AdminPingResponse> {
    Json(AdminPingResponse {
        status: "ok",
        admin_id: admin.admin_id.to_string(),
        email: admin.email,
        token_jti: admin.jti,
    })
}
