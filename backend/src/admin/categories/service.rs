use sqlx::PgPool;
use uuid::Uuid;

use crate::models::category::Category;

#[derive(Debug, Clone)]
pub struct NewCategory {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone)]
pub struct CategoryChanges {
    pub name: String,
    pub slug: String,
}

pub async fn create_category(pool: &PgPool, new_category: NewCategory) -> Result<Category, sqlx::Error> {
    sqlx::query_as::<_, Category>(
        "INSERT INTO categories (name, slug) VALUES ($1, $2) RETURNING id, name, slug, created_at",
    )
    .bind(new_category.name)
    .bind(new_category.slug)
    .fetch_one(pool)
    .await
}

pub async fn list_categories(pool: &PgPool) -> Result<Vec<Category>, sqlx::Error> {
    sqlx::query_as::<_, Category>(
        "SELECT id, name, slug, created_at FROM categories ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await
}

pub async fn get_category(pool: &PgPool, id: Uuid) -> Result<Option<Category>, sqlx::Error> {
    sqlx::query_as::<_, Category>(
        "SELECT id, name, slug, created_at FROM categories WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn update_category(
    pool: &PgPool,
    id: Uuid,
    changes: CategoryChanges,
) -> Result<Option<Category>, sqlx::Error> {
    sqlx::query_as::<_, Category>(
        "UPDATE categories SET name = $1, slug = $2 WHERE id = $3 RETURNING id, name, slug, created_at",
    )
    .bind(changes.name)
    .bind(changes.slug)
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn delete_category(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM categories WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}
