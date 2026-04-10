use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{auth::jwt, state::AppState};

#[derive(Debug, Clone)]
pub struct AuthenticatedAdmin {
    pub admin_id: Uuid,
    pub email: String,
    pub jti: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn require_admin_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let token = extract_bearer_token(&request).ok_or_else(|| unauthorized("missing Bearer token"))?;

    let claims = jwt::decode_token(&token, &state.jwt_config)
        .map_err(|_| unauthorized("invalid token"))?;

    let admin_id = Uuid::parse_str(&claims.sub).map_err(|_| unauthorized("invalid token subject"))?;

    let is_revoked = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM revoked_jwts WHERE jti = $1 AND expires_at > NOW() LIMIT 1",
    )
    .bind(&claims.jti)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| internal("failed to verify token status"))?
    .is_some();

    if is_revoked {
        return Err(unauthorized("token has been revoked"));
    }

    request.extensions_mut().insert(AuthenticatedAdmin {
        admin_id,
        email: claims.email,
        jti: claims.jti,
    });

    Ok(next.run(request).await)
}

fn extract_bearer_token(request: &Request) -> Option<String> {
    let header = request.headers().get("authorization")?.to_str().ok()?;
    let (scheme, token) = header.split_once(' ')?;

    if !scheme.eq_ignore_ascii_case("bearer") || token.trim().is_empty() {
        return None;
    }

    Some(token.trim().to_string())
}

fn unauthorized(message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: message.to_string(),
        }),
    )
        .into_response()
}

fn internal(message: &str) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: message.to_string(),
        }),
    )
        .into_response()
}
