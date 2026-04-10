use axum::{
    extract::Extension,
    routing::{get, post, put},
    Json, Router,
};
use serde::Serialize;

use crate::{
    admin::{
        categories::handlers as category_handlers, posts::handlers as post_handlers,
        tags::handlers as tag_handlers, uploads::handlers as upload_handlers,
    },
    auth::middleware::AuthenticatedAdmin,
};

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
        .route(
            "/api/admin/tags",
            post(tag_handlers::create_tag).get(tag_handlers::list_tags),
        )
        .route(
            "/api/admin/tags/:id",
            get(tag_handlers::get_tag)
                .put(tag_handlers::update_tag)
                .delete(tag_handlers::delete_tag),
        )
        .route(
            "/api/admin/posts",
            post(post_handlers::create_post).get(post_handlers::list_posts),
        )
        .route("/api/admin/posts/slug/:slug", get(post_handlers::get_post_by_slug))
        .route(
            "/api/admin/posts/:id",
            put(post_handlers::update_post).delete(post_handlers::delete_post),
        )
        .route("/api/admin/upload", post(upload_handlers::upload_image))
}

async fn admin_ping(Extension(admin): Extension<AuthenticatedAdmin>) -> Json<AdminPingResponse> {
    Json(AdminPingResponse {
        status: "ok",
        admin_id: admin.admin_id.to_string(),
        email: admin.email,
        token_jti: admin.jti,
    })
}
