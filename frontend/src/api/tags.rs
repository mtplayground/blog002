use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct UpsertTagRequest {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TagDto {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub created_at: String,
}

#[cfg(target_arch = "wasm32")]
pub async fn list_tags(token: &str) -> Result<Vec<TagDto>, String> {
    use gloo_net::http::Request;

    let response = Request::get("/api/admin/tags")
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|err| format!("failed to fetch tags: {err}"))?;

    if !response.ok() {
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "failed to fetch tags".to_string());
        return Err(message);
    }

    response
        .json::<Vec<TagDto>>()
        .await
        .map_err(|err| format!("invalid tags payload: {err}"))
}

#[cfg(target_arch = "wasm32")]
pub async fn create_tag(token: &str, payload: &UpsertTagRequest) -> Result<TagDto, String> {
    use gloo_net::http::Request;

    let response = Request::post("/api/admin/tags")
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .json(payload)
        .map_err(|err| format!("failed to serialize tag payload: {err}"))?
        .send()
        .await
        .map_err(|err| format!("failed to create tag: {err}"))?;

    if !response.ok() {
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "failed to create tag".to_string());
        return Err(message);
    }

    response
        .json::<TagDto>()
        .await
        .map_err(|err| format!("invalid tag payload: {err}"))
}

#[cfg(target_arch = "wasm32")]
pub async fn update_tag(token: &str, id: &str, payload: &UpsertTagRequest) -> Result<TagDto, String> {
    use gloo_net::http::Request;

    let response = Request::put(&format!("/api/admin/tags/{id}"))
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .json(payload)
        .map_err(|err| format!("failed to serialize tag payload: {err}"))?
        .send()
        .await
        .map_err(|err| format!("failed to update tag: {err}"))?;

    if !response.ok() {
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "failed to update tag".to_string());
        return Err(message);
    }

    response
        .json::<TagDto>()
        .await
        .map_err(|err| format!("invalid tag payload: {err}"))
}

#[cfg(target_arch = "wasm32")]
pub async fn delete_tag(token: &str, id: &str) -> Result<(), String> {
    use gloo_net::http::Request;

    let response = Request::delete(&format!("/api/admin/tags/{id}"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|err| format!("failed to delete tag: {err}"))?;

    if !response.ok() {
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "failed to delete tag".to_string());
        return Err(message);
    }

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn list_tags(_token: &str) -> Result<Vec<TagDto>, String> {
    Err("tag API client is only available in the browser runtime".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_tag(_token: &str, _payload: &UpsertTagRequest) -> Result<TagDto, String> {
    Err("tag API client is only available in the browser runtime".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn update_tag(_token: &str, _id: &str, _payload: &UpsertTagRequest) -> Result<TagDto, String> {
    Err("tag API client is only available in the browser runtime".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_tag(_token: &str, _id: &str) -> Result<(), String> {
    Err("tag API client is only available in the browser runtime".to_string())
}
