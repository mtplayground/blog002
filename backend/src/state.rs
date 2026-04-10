use sqlx::PgPool;

use crate::auth::jwt::JwtConfig;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub jwt_config: JwtConfig,
}
