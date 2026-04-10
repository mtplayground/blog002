use sqlx::PgPool;

use crate::{auth::jwt::JwtConfig, uploads::service::UploadService};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub jwt_config: JwtConfig,
    pub upload_service: UploadService,
}
