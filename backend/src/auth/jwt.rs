use std::env;

use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expires_minutes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,
}

impl JwtConfig {
    pub fn from_env() -> Result<Self> {
        let secret = env::var("JWT_SECRET").context("JWT_SECRET is required")?;

        if secret.len() < 32 {
            anyhow::bail!("JWT_SECRET must be at least 32 characters long");
        }

        let expires_minutes = env::var("JWT_EXPIRES_MINUTES")
            .ok()
            .unwrap_or_else(|| "60".to_string())
            .parse::<i64>()
            .context("JWT_EXPIRES_MINUTES must be a positive integer")?;

        if expires_minutes <= 0 {
            anyhow::bail!("JWT_EXPIRES_MINUTES must be greater than 0");
        }

        Ok(Self {
            secret,
            expires_minutes,
        })
    }
}

pub fn issue_token(admin_id: Uuid, email: &str, config: &JwtConfig) -> Result<String> {
    let issued_at = Utc::now();
    let expires_at = issued_at + Duration::minutes(config.expires_minutes);

    let claims = Claims {
        sub: admin_id.to_string(),
        email: email.to_string(),
        iat: issued_at.timestamp() as usize,
        exp: expires_at.timestamp() as usize,
        jti: Uuid::new_v4().to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .context("failed to encode JWT")?;

    Ok(token)
}

pub fn decode_token(token: &str, config: &JwtConfig) -> Result<Claims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &validation,
    )
    .context("failed to decode JWT")?;

    Ok(token_data.claims)
}
