use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct UpsertCategoryRequest {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CategoryDto {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub created_at: String,
}

#[cfg(target_arch = "wasm32")]
pub async fn list_categories(token: &str) -> Result<Vec<CategoryDto>, String> {
    use gloo_net::http::Request;

    let response = Request::get("/api/admin/categories")
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|err| format!("failed to fetch categories: {err}"))?;

    if !response.ok() {
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "failed to fetch categories".to_string());
        return Err(message);
    }

    response
        .json::<Vec<CategoryDto>>()
        .await
        .map_err(|err| format!("invalid categories payload: {err}"))
}

#[cfg(target_arch = "wasm32")]
pub async fn create_category(token: &str, payload: &UpsertCategoryRequest) -> Result<CategoryDto, String> {
    use gloo_net::http::Request;

    let response = Request::post("/api/admin/categories")
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .json(payload)
        .map_err(|err| format!("failed to serialize category payload: {err}"))?
        .send()
        .await
        .map_err(|err| format!("failed to create category: {err}"))?;

    if !response.ok() {
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "failed to create category".to_string());
        return Err(message);
    }

    response
        .json::<CategoryDto>()
        .await
        .map_err(|err| format!("invalid category payload: {err}"))
}

#[cfg(target_arch = "wasm32")]
pub async fn update_category(
    token: &str,
    id: &str,
    payload: &UpsertCategoryRequest,
) -> Result<CategoryDto, String> {
    use gloo_net::http::Request;

    let response = Request::put(&format!("/api/admin/categories/{id}"))
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .json(payload)
        .map_err(|err| format!("failed to serialize category payload: {err}"))?
        .send()
        .await
        .map_err(|err| format!("failed to update category: {err}"))?;

    if !response.ok() {
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "failed to update category".to_string());
        return Err(message);
    }

    response
        .json::<CategoryDto>()
        .await
        .map_err(|err| format!("invalid category payload: {err}"))
}

#[cfg(target_arch = "wasm32")]
pub async fn delete_category(token: &str, id: &str) -> Result<(), String> {
    use gloo_net::http::Request;

    let response = Request::delete(&format!("/api/admin/categories/{id}"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|err| format!("failed to delete category: {err}"))?;

    if !response.ok() {
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "failed to delete category".to_string());
        return Err(message);
    }

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn list_categories(_token: &str) -> Result<Vec<CategoryDto>, String> {
    Err("category API client is only available in the browser runtime".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_category(_token: &str, _payload: &UpsertCategoryRequest) -> Result<CategoryDto, String> {
    Err("category API client is only available in the browser runtime".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn update_category(
    _token: &str,
    _id: &str,
    _payload: &UpsertCategoryRequest,
) -> Result<CategoryDto, String> {
    Err("category API client is only available in the browser runtime".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_category(_token: &str, _id: &str) -> Result<(), String> {
    Err("category API client is only available in the browser runtime".to_string())
}
