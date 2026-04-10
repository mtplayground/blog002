use axum::{
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    auth::{jwt, password},
    models::admin::Admin,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub token_type: &'static str,
    pub expires_at: i64,
}

#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub message: &'static str,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    if payload.email.trim().is_empty() || payload.password.is_empty() {
        return Err(ApiError::bad_request("email and password are required"));
    }

    let admin = sqlx::query_as::<_, Admin>(
        "SELECT id, email, password_hash, created_at FROM admins WHERE email = $1",
    )
    .bind(payload.email.trim().to_lowercase())
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| ApiError::internal("failed to fetch admin credentials"))?;

    let admin = match admin {
        Some(admin) => admin,
        None => return Err(ApiError::unauthorized("invalid email or password")),
    };

    let is_valid = password::verify_password(&payload.password, &admin.password_hash)
        .map_err(|_| ApiError::internal("failed to validate password"))?;

    if !is_valid {
        return Err(ApiError::unauthorized("invalid email or password"));
    }

    let token = jwt::issue_token(admin.id, &admin.email, &state.jwt_config)
        .map_err(|_| ApiError::internal("failed to issue JWT"))?;

    let claims = jwt::decode_token(&token, &state.jwt_config)
        .map_err(|_| ApiError::internal("failed to decode generated JWT"))?;

    Ok(Json(LoginResponse {
        token,
        token_type: "Bearer",
        expires_at: claims.exp as i64,
    }))
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<LogoutResponse>, ApiError> {
    let token = extract_bearer_token(&headers)
        .ok_or_else(|| ApiError::unauthorized("missing Bearer token"))?;

    let claims = jwt::decode_token(token, &state.jwt_config)
        .map_err(|_| ApiError::unauthorized("invalid token"))?;

    let expires_at = DateTime::<Utc>::from_timestamp(claims.exp as i64, 0)
        .ok_or_else(|| ApiError::unauthorized("invalid token expiration"))?;

    sqlx::query("INSERT INTO revoked_jwts (jti, expires_at) VALUES ($1, $2) ON CONFLICT (jti) DO NOTHING")
        .bind(claims.jti)
        .bind(expires_at)
        .execute(&state.db_pool)
        .await
        .map_err(|_| ApiError::internal("failed to revoke token"))?;

    Ok(Json(LogoutResponse {
        message: "session invalidated",
    }))
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    let header = headers.get("authorization")?.to_str().ok()?;
    let (scheme, token) = header.split_once(' ')?;

    if !scheme.eq_ignore_ascii_case("bearer") || token.trim().is_empty() {
        return None;
    }

    Some(token.trim())
}

pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(message: &str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.to_string(),
        }
    }

    fn unauthorized(message: &str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.to_string(),
        }
    }

    fn internal(message: &str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorResponse {
                error: self.message,
            }),
        )
            .into_response()
    }
}
