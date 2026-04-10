use axum::{routing::post, Router};

use crate::{auth::handlers, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/logout", post(handlers::logout))
}
