use sqlx::PgPool;
use uuid::Uuid;

use crate::models::tag::Tag;

#[derive(Debug, Clone)]
pub struct NewTag {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone)]
pub struct TagChanges {
    pub name: String,
    pub slug: String,
}

pub async fn create_tag(pool: &PgPool, new_tag: NewTag) -> Result<Tag, sqlx::Error> {
    sqlx::query_as::<_, Tag>(
        "INSERT INTO tags (name, slug) VALUES ($1, $2) RETURNING id, name, slug, created_at",
    )
    .bind(new_tag.name)
    .bind(new_tag.slug)
    .fetch_one(pool)
    .await
}

pub async fn list_tags(pool: &PgPool) -> Result<Vec<Tag>, sqlx::Error> {
    sqlx::query_as::<_, Tag>("SELECT id, name, slug, created_at FROM tags ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn get_tag(pool: &PgPool, id: Uuid) -> Result<Option<Tag>, sqlx::Error> {
    sqlx::query_as::<_, Tag>("SELECT id, name, slug, created_at FROM tags WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn update_tag(pool: &PgPool, id: Uuid, changes: TagChanges) -> Result<Option<Tag>, sqlx::Error> {
    sqlx::query_as::<_, Tag>(
        "UPDATE tags SET name = $1, slug = $2 WHERE id = $3 RETURNING id, name, slug, created_at",
    )
    .bind(changes.name)
    .bind(changes.slug)
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn delete_tag(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM tags WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}
