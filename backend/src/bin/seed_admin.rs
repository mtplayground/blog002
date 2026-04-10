use std::env;

use anyhow::{bail, Context, Result};
use backend::{auth::password, db};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let email = env::var("ADMIN_EMAIL").context("ADMIN_EMAIL is required")?;
    let password = env::var("ADMIN_PASSWORD").context("ADMIN_PASSWORD is required")?;

    validate_seed_input(&email, &password)?;

    let db_settings = db::DatabaseSettings::from_env()?;
    let pool = db::connect(&db_settings).await?;
    db::run_migrations(&pool).await?;

    let password_hash = password::hash_password(&password)?;

    let result = sqlx::query(
        "INSERT INTO admins (email, password_hash) VALUES ($1, $2) ON CONFLICT (email) DO NOTHING",
    )
    .bind(&email)
    .bind(&password_hash)
    .execute(&pool)
    .await
    .context("failed to execute admin seed insert")?;

    if result.rows_affected() == 0 {
        println!("admin already exists for email: {email}");
    } else {
        println!("admin created for email: {email}");
    }

    Ok(())
}

fn validate_seed_input(email: &str, password: &str) -> Result<()> {
    if !email.contains('@') {
        bail!("ADMIN_EMAIL must be a valid email address");
    }

    if password.len() < 12 {
        bail!("ADMIN_PASSWORD must be at least 12 characters long");
    }

    Ok(())
}
