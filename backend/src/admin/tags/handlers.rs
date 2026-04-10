use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{admin::tags::service, state::AppState};

#[derive(Debug, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTagRequest {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize)]
pub struct TagResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn create_tag(
    State(state): State<AppState>,
    Json(payload): Json<CreateTagRequest>,
) -> Result<(StatusCode, Json<TagResponse>), ApiError> {
    let input = validate_tag_fields(payload.name, payload.slug)?;

    let tag = service::create_tag(&state.db_pool, input)
        .await
        .map_err(map_sqlx_error)?;

    Ok((StatusCode::CREATED, Json(to_response(tag))))
}

pub async fn list_tags(State(state): State<AppState>) -> Result<Json<Vec<TagResponse>>, ApiError> {
    let tags = service::list_tags(&state.db_pool)
        .await
        .map_err(map_sqlx_error)?;

    let payload = tags.into_iter().map(to_response).collect::<Vec<_>>();

    Ok(Json(payload))
}

pub async fn get_tag(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<TagResponse>, ApiError> {
    let Some(tag) = service::get_tag(&state.db_pool, id)
        .await
        .map_err(map_sqlx_error)?
    else {
        return Err(ApiError::not_found("tag not found"));
    };

    Ok(Json(to_response(tag)))
}

pub async fn update_tag(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTagRequest>,
) -> Result<Json<TagResponse>, ApiError> {
    let changes = validate_tag_changes(payload.name, payload.slug)?;

    let Some(tag) = service::update_tag(&state.db_pool, id, changes)
        .await
        .map_err(map_sqlx_error)?
    else {
        return Err(ApiError::not_found("tag not found"));
    };

    Ok(Json(to_response(tag)))
}

pub async fn delete_tag(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let deleted = service::delete_tag(&state.db_pool, id)
        .await
        .map_err(map_sqlx_error)?;

    if !deleted {
        return Err(ApiError::not_found("tag not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}

fn validate_tag_fields(name: String, slug: String) -> Result<service::NewTag, ApiError> {
    let name = normalize_name(name)?;
    let slug = normalize_slug(slug)?;

    Ok(service::NewTag { name, slug })
}

fn validate_tag_changes(name: String, slug: String) -> Result<service::TagChanges, ApiError> {
    let name = normalize_name(name)?;
    let slug = normalize_slug(slug)?;

    Ok(service::TagChanges { name, slug })
}

fn normalize_name(name: String) -> Result<String, ApiError> {
    let normalized = name.trim();

    if normalized.is_empty() {
        return Err(ApiError::bad_request("name is required"));
    }

    if normalized.len() > 120 {
        return Err(ApiError::bad_request("name must be 120 characters or fewer"));
    }

    Ok(normalized.to_string())
}

fn normalize_slug(slug: String) -> Result<String, ApiError> {
    let normalized = slug.trim().to_lowercase();

    if normalized.is_empty() {
        return Err(ApiError::bad_request("slug is required"));
    }

    if normalized.len() > 120 {
        return Err(ApiError::bad_request("slug must be 120 characters or fewer"));
    }

    let is_valid = normalized
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');

    if !is_valid {
        return Err(ApiError::bad_request(
            "slug may only contain lowercase letters, numbers, and hyphens",
        ));
    }

    Ok(normalized)
}

fn to_response(tag: crate::models::tag::Tag) -> TagResponse {
    TagResponse {
        id: tag.id,
        name: tag.name,
        slug: tag.slug,
        created_at: tag.created_at,
    }
}

fn map_sqlx_error(error: sqlx::Error) -> ApiError {
    if let sqlx::Error::Database(db_error) = &error {
        if db_error.code().as_deref() == Some("23505") {
            return ApiError::conflict("tag name or slug already exists");
        }
    }

    ApiError::internal("database operation failed")
}

struct ApiError {
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

    fn not_found(message: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.to_string(),
        }
    }

    fn conflict(message: &str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
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
