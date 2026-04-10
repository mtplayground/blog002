use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub token_type: String,
    pub expires_at: i64,
}

#[cfg(target_arch = "wasm32")]
pub async fn login(payload: &LoginRequest) -> Result<LoginResponse, String> {
    use gloo_net::http::Request;

    let response = Request::post("/api/auth/login")
        .header("Content-Type", "application/json")
        .json(payload)
        .map_err(|err| format!("failed to serialize login payload: {err}"))?
        .send()
        .await
        .map_err(|err| format!("login request failed: {err}"))?;

    if !response.ok() {
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "authentication failed".to_string());
        return Err(message);
    }

    response
        .json::<LoginResponse>()
        .await
        .map_err(|err| format!("failed to parse login response: {err}"))
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn login(_payload: &LoginRequest) -> Result<LoginResponse, String> {
    Err("login API client is only available in the browser runtime".to_string())
}
