use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PostCategoryDto {
    pub id: String,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostTagDto {
    pub id: String,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostDto {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub featured_image_url: Option<String>,
    pub category: PostCategoryDto,
    pub status: String,
    pub published_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub tags: Vec<PostTagDto>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaginatedPostsDto {
    pub items: Vec<PostDto>,
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListPostsQuery {
    pub page: u32,
    pub per_page: u32,
}

#[cfg(target_arch = "wasm32")]
pub async fn list_posts(token: &str, query: &ListPostsQuery) -> Result<PaginatedPostsDto, String> {
    use gloo_net::http::Request;

    let response = Request::get(&format!(
        "/api/admin/posts?page={}&per_page={}",
        query.page, query.per_page
    ))
    .header("Authorization", &format!("Bearer {token}"))
    .send()
    .await
    .map_err(|err| format!("failed to fetch posts: {err}"))?;

    if !response.ok() {
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "failed to fetch posts".to_string());
        return Err(message);
    }

    response
        .json::<PaginatedPostsDto>()
        .await
        .map_err(|err| format!("invalid posts payload: {err}"))
}

#[cfg(target_arch = "wasm32")]
pub async fn delete_post(token: &str, id: &str) -> Result<(), String> {
    use gloo_net::http::Request;

    let response = Request::delete(&format!("/api/admin/posts/{id}"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|err| format!("failed to delete post: {err}"))?;

    if !response.ok() {
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "failed to delete post".to_string());
        return Err(message);
    }

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn list_posts(_token: &str, _query: &ListPostsQuery) -> Result<PaginatedPostsDto, String> {
    Err("posts API client is only available in the browser runtime".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_post(_token: &str, _id: &str) -> Result<(), String> {
    Err("posts API client is only available in the browser runtime".to_string())
}
