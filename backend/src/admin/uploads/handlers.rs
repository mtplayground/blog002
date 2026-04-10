use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use uuid::Uuid;

use crate::state::AppState;

const MAX_FILE_SIZE_BYTES: usize = 10 * 1024 * 1024;

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub url: String,
    pub key: String,
    pub content_type: String,
    pub size: usize,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn upload_image(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<UploadResponse>), ApiError> {
    let mut uploaded = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::bad_request("invalid multipart payload"))?
    {
        if field.name() != Some("file") {
            continue;
        }

        let content_type = field
            .content_type()
            .map(|value| value.to_string())
            .ok_or_else(|| ApiError::bad_request("file content type is required"))?;

        validate_content_type(&content_type)?;

        let file_name = field
            .file_name()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "upload.bin".to_string());

        let bytes = field
            .bytes()
            .await
            .map_err(|_| ApiError::bad_request("failed to read uploaded file"))?;

        if bytes.is_empty() {
            return Err(ApiError::bad_request("uploaded file is empty"));
        }

        if bytes.len() > MAX_FILE_SIZE_BYTES {
            return Err(ApiError::bad_request("uploaded file exceeds 10MB limit"));
        }

        let key = build_object_key(&file_name, &content_type);
        let url = state
            .upload_service
            .upload_bytes(&key, bytes.to_vec(), Some(&content_type))
            .await
            .map_err(|_| ApiError::internal("failed to upload file to object storage"))?;

        uploaded = Some(UploadResponse {
            url,
            key,
            content_type,
            size: bytes.len(),
        });

        break;
    }

    let Some(response) = uploaded else {
        return Err(ApiError::bad_request(
            "multipart field 'file' is required for upload",
        ));
    };

    Ok((StatusCode::CREATED, Json(response)))
}

fn validate_content_type(content_type: &str) -> Result<(), ApiError> {
    let allowed = [
        "image/jpeg",
        "image/png",
        "image/gif",
        "image/webp",
        "image/svg+xml",
    ];

    if !allowed.contains(&content_type) {
        return Err(ApiError::bad_request(
            "unsupported file type; allowed: jpeg, png, gif, webp, svg",
        ));
    }

    Ok(())
}

fn build_object_key(file_name: &str, content_type: &str) -> String {
    let extension = extension_from_content_type(content_type)
        .or_else(|| file_name.rsplit('.').next().map(|value| value.to_lowercase()))
        .unwrap_or_else(|| "bin".to_string());

    format!("uploads/{}.{}", Uuid::new_v4(), sanitize_extension(&extension))
}

fn extension_from_content_type(content_type: &str) -> Option<String> {
    let extension = match content_type {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        _ => return None,
    };

    Some(extension.to_string())
}

fn sanitize_extension(value: &str) -> String {
    let normalized = value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>();

    if normalized.is_empty() {
        "bin".to_string()
    } else {
        normalized
    }
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
