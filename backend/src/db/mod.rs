use std::{env, time::Duration};

use anyhow::{Context, Result};
use sqlx::{postgres::PgPoolOptions, PgPool};

#[derive(Debug, Clone)]
pub struct DatabaseSettings {
    pub database_url: String,
    pub max_connections: u32,
    pub connect_timeout_secs: u64,
}

impl DatabaseSettings {
    pub fn from_env() -> Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .context("DATABASE_URL is required to initialize the database connection")?;

        let max_connections = env::var("DB_MAX_CONNECTIONS")
            .ok()
            .unwrap_or_else(|| "10".to_string())
            .parse::<u32>()
            .context("DB_MAX_CONNECTIONS must be a positive integer")?;

        let connect_timeout_secs = env::var("DB_CONNECT_TIMEOUT_SECS")
            .ok()
            .unwrap_or_else(|| "5".to_string())
            .parse::<u64>()
            .context("DB_CONNECT_TIMEOUT_SECS must be a positive integer")?;

        Ok(Self {
            database_url,
            max_connections,
            connect_timeout_secs,
        })
    }
}

pub async fn connect(settings: &DatabaseSettings) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(settings.max_connections)
        .acquire_timeout(Duration::from_secs(settings.connect_timeout_secs))
        .connect(&settings.database_url)
        .await
        .context("failed to connect to PostgreSQL")?;

    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("../migrations")
        .run(pool)
        .await
        .context("failed to run SQLx migrations")?;

    Ok(())
}
