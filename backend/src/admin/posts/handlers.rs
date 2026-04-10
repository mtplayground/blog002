use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

use crate::{admin::posts::service, state::AppState};

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub slug: String,
    pub body: String,
    pub featured_image_url: Option<String>,
    pub category_id: Uuid,
    pub status: String,
    pub published_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub tag_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePostRequest {
    pub title: String,
    pub slug: String,
    pub body: String,
    pub featured_image_url: Option<String>,
    pub category_id: Uuid,
    pub status: String,
    pub published_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub tag_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ListPostsQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct SlugPath {
    pub slug: String,
}

#[derive(Debug, Serialize)]
pub struct PostCategoryResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize)]
pub struct PostTagResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize)]
pub struct PostResponse {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub featured_image_url: Option<String>,
    pub category: PostCategoryResponse,
    pub status: String,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<PostTagResponse>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedPostsResponse {
    pub items: Vec<PostResponse>,
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn create_post(
    State(state): State<AppState>,
    Json(payload): Json<CreatePostRequest>,
) -> Result<(StatusCode, Json<PostResponse>), ApiError> {
    let input = validate_new_post(payload)?;

    let post = service::create_post(&state.db_pool, input)
        .await
        .map_err(map_sqlx_error)?;

    Ok((StatusCode::CREATED, Json(to_response(post))))
}

pub async fn list_posts(
    State(state): State<AppState>,
    Query(query): Query<ListPostsQuery>,
) -> Result<Json<PaginatedPostsResponse>, ApiError> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(10);

    if page == 0 {
        return Err(ApiError::bad_request("page must be greater than 0"));
    }

    if per_page == 0 || per_page > 100 {
        return Err(ApiError::bad_request(
            "per_page must be between 1 and 100",
        ));
    }

    let result = service::list_posts(&state.db_pool, service::PostListOptions { page, per_page })
        .await
        .map_err(map_sqlx_error)?;

    Ok(Json(PaginatedPostsResponse {
        items: result.items.into_iter().map(to_response).collect(),
        page: result.page,
        per_page: result.per_page,
        total: result.total,
    }))
}

pub async fn get_post_by_slug(
    State(state): State<AppState>,
    Path(path): Path<SlugPath>,
) -> Result<Json<PostResponse>, ApiError> {
    let slug = normalize_slug(path.slug)?;

    let Some(post) = service::get_post_by_slug(&state.db_pool, &slug)
        .await
        .map_err(map_sqlx_error)?
    else {
        return Err(ApiError::not_found("post not found"));
    };

    Ok(Json(to_response(post)))
}

pub async fn update_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePostRequest>,
) -> Result<Json<PostResponse>, ApiError> {
    let changes = validate_post_changes(payload)?;

    let Some(post) = service::update_post(&state.db_pool, id, changes)
        .await
        .map_err(map_sqlx_error)?
    else {
        return Err(ApiError::not_found("post not found"));
    };

    Ok(Json(to_response(post)))
}

pub async fn delete_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let deleted = service::delete_post(&state.db_pool, id)
        .await
        .map_err(map_sqlx_error)?;

    if !deleted {
        return Err(ApiError::not_found("post not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}

fn validate_new_post(payload: CreatePostRequest) -> Result<service::NewPost, ApiError> {
    let title = normalize_title(payload.title)?;
    let slug = normalize_slug(payload.slug)?;
    let body = normalize_body(payload.body)?;
    let featured_image_url = normalize_featured_image_url(payload.featured_image_url)?;
    let status = normalize_status(payload.status)?;
    let published_at = normalize_published_at(&status, payload.published_at);
    let tag_ids = normalize_tag_ids(payload.tag_ids);

    Ok(service::NewPost {
        title,
        slug,
        body,
        featured_image_url,
        category_id: payload.category_id,
        status,
        published_at,
        tag_ids,
    })
}

fn validate_post_changes(payload: UpdatePostRequest) -> Result<service::PostChanges, ApiError> {
    let title = normalize_title(payload.title)?;
    let slug = normalize_slug(payload.slug)?;
    let body = normalize_body(payload.body)?;
    let featured_image_url = normalize_featured_image_url(payload.featured_image_url)?;
    let status = normalize_status(payload.status)?;
    let published_at = normalize_published_at(&status, payload.published_at);
    let tag_ids = normalize_tag_ids(payload.tag_ids);

    Ok(service::PostChanges {
        title,
        slug,
        body,
        featured_image_url,
        category_id: payload.category_id,
        status,
        published_at,
        tag_ids,
    })
}

fn normalize_title(value: String) -> Result<String, ApiError> {
    let normalized = value.trim();

    if normalized.is_empty() {
        return Err(ApiError::bad_request("title is required"));
    }

    if normalized.len() > 255 {
        return Err(ApiError::bad_request("title must be 255 characters or fewer"));
    }

    Ok(normalized.to_string())
}

fn normalize_slug(value: String) -> Result<String, ApiError> {
    let normalized = value.trim().to_lowercase();

    if normalized.is_empty() {
        return Err(ApiError::bad_request("slug is required"));
    }

    if normalized.len() > 160 {
        return Err(ApiError::bad_request("slug must be 160 characters or fewer"));
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

fn normalize_body(value: String) -> Result<String, ApiError> {
    let normalized = value.trim();

    if normalized.is_empty() {
        return Err(ApiError::bad_request("body is required"));
    }

    Ok(normalized.to_string())
}

fn normalize_featured_image_url(value: Option<String>) -> Result<Option<String>, ApiError> {
    let Some(raw) = value else {
        return Ok(None);
    };

    let normalized = raw.trim();
    if normalized.is_empty() {
        return Ok(None);
    }

    if normalized.len() > 2048 {
        return Err(ApiError::bad_request(
            "featured_image_url must be 2048 characters or fewer",
        ));
    }

    Ok(Some(normalized.to_string()))
}

fn normalize_status(value: String) -> Result<String, ApiError> {
    let normalized = value.trim().to_lowercase();

    match normalized.as_str() {
        "draft" | "published" | "archived" => Ok(normalized),
        _ => Err(ApiError::bad_request(
            "status must be one of: draft, published, archived",
        )),
    }
}

fn normalize_published_at(status: &str, published_at: Option<DateTime<Utc>>) -> Option<DateTime<Utc>> {
    if status == "published" {
        published_at.or_else(|| Some(Utc::now()))
    } else {
        published_at
    }
}

fn normalize_tag_ids(value: Vec<Uuid>) -> Vec<Uuid> {
    let mut seen = HashSet::<Uuid>::new();
    let mut unique = Vec::<Uuid>::new();

    for tag_id in value {
        if seen.insert(tag_id) {
            unique.push(tag_id);
        }
    }

    unique
}

fn to_response(post: service::PostDetails) -> PostResponse {
    PostResponse {
        id: post.id,
        title: post.title,
        slug: post.slug,
        body: post.body,
        featured_image_url: post.featured_image_url,
        category: PostCategoryResponse {
            id: post.category.id,
            name: post.category.name,
            slug: post.category.slug,
        },
        status: post.status,
        published_at: post.published_at,
        created_at: post.created_at,
        updated_at: post.updated_at,
        tags: post
            .tags
            .into_iter()
            .map(|tag| PostTagResponse {
                id: tag.id,
                name: tag.name,
                slug: tag.slug,
            })
            .collect(),
    }
}

fn map_sqlx_error(error: sqlx::Error) -> ApiError {
    if let sqlx::Error::Database(db_error) = &error {
        match db_error.code().as_deref() {
            Some("23505") => return ApiError::conflict("post slug already exists"),
            Some("23503") => {
                return ApiError::bad_request("invalid category_id or tag_ids reference")
            }
            _ => {}
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
