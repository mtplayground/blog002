use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{admin::categories::service, state::AppState};

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize)]
pub struct CategoryResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn create_category(
    State(state): State<AppState>,
    Json(payload): Json<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<CategoryResponse>), ApiError> {
    let input = validate_category_fields(payload.name, payload.slug)?;

    let category = service::create_category(&state.db_pool, input)
        .await
        .map_err(map_sqlx_error)?;

    Ok((StatusCode::CREATED, Json(to_response(category))))
}

pub async fn list_categories(
    State(state): State<AppState>,
) -> Result<Json<Vec<CategoryResponse>>, ApiError> {
    let categories = service::list_categories(&state.db_pool)
        .await
        .map_err(map_sqlx_error)?;

    let payload = categories.into_iter().map(to_response).collect::<Vec<_>>();

    Ok(Json(payload))
}

pub async fn get_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CategoryResponse>, ApiError> {
    let Some(category) = service::get_category(&state.db_pool, id)
        .await
        .map_err(map_sqlx_error)?
    else {
        return Err(ApiError::not_found("category not found"));
    };

    Ok(Json(to_response(category)))
}

pub async fn update_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCategoryRequest>,
) -> Result<Json<CategoryResponse>, ApiError> {
    let changes = validate_category_changes(payload.name, payload.slug)?;

    let Some(category) = service::update_category(&state.db_pool, id, changes)
        .await
        .map_err(map_sqlx_error)?
    else {
        return Err(ApiError::not_found("category not found"));
    };

    Ok(Json(to_response(category)))
}

pub async fn delete_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let deleted = service::delete_category(&state.db_pool, id)
        .await
        .map_err(map_sqlx_error)?;

    if !deleted {
        return Err(ApiError::not_found("category not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}

fn validate_category_fields(name: String, slug: String) -> Result<service::NewCategory, ApiError> {
    let name = normalize_name(name)?;
    let slug = normalize_slug(slug)?;

    Ok(service::NewCategory { name, slug })
}

fn validate_category_changes(
    name: String,
    slug: String,
) -> Result<service::CategoryChanges, ApiError> {
    let name = normalize_name(name)?;
    let slug = normalize_slug(slug)?;

    Ok(service::CategoryChanges { name, slug })
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

fn to_response(category: crate::models::category::Category) -> CategoryResponse {
    CategoryResponse {
        id: category.id,
        name: category.name,
        slug: category.slug,
        created_at: category.created_at,
    }
}

fn map_sqlx_error(error: sqlx::Error) -> ApiError {
    if let sqlx::Error::Database(db_error) = &error {
        if db_error.code().as_deref() == Some("23505") {
            return ApiError::conflict("category name or slug already exists");
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
