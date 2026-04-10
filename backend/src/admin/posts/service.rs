use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CategorySummary {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone)]
pub struct TagSummary {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone)]
pub struct PostDetails {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub featured_image_url: Option<String>,
    pub category: CategorySummary,
    pub status: String,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<TagSummary>,
}

#[derive(Debug, Clone)]
pub struct NewPost {
    pub title: String,
    pub slug: String,
    pub body: String,
    pub featured_image_url: Option<String>,
    pub category_id: Uuid,
    pub status: String,
    pub published_at: Option<DateTime<Utc>>,
    pub tag_ids: Vec<Uuid>,
}

#[derive(Debug, Clone)]
pub struct PostChanges {
    pub title: String,
    pub slug: String,
    pub body: String,
    pub featured_image_url: Option<String>,
    pub category_id: Uuid,
    pub status: String,
    pub published_at: Option<DateTime<Utc>>,
    pub tag_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Copy)]
pub struct PostListOptions {
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Clone)]
pub struct PaginatedPosts {
    pub items: Vec<PostDetails>,
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
}

pub async fn create_post(pool: &PgPool, new_post: NewPost) -> Result<PostDetails, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let post_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO posts (
            title,
            slug,
            body,
            featured_image_url,
            category_id,
            status,
            published_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
    )
    .bind(new_post.title)
    .bind(new_post.slug)
    .bind(new_post.body)
    .bind(new_post.featured_image_url)
    .bind(new_post.category_id)
    .bind(new_post.status)
    .bind(new_post.published_at)
    .fetch_one(&mut *tx)
    .await?;

    replace_post_tags(&mut tx, post_id, &new_post.tag_ids).await?;
    tx.commit().await?;

    fetch_post_details_by_id(pool, post_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn list_posts(pool: &PgPool, options: PostListOptions) -> Result<PaginatedPosts, sqlx::Error> {
    let offset = (options.page.saturating_sub(1) * options.per_page) as i64;
    let limit = options.per_page as i64;

    let rows = sqlx::query(
        r#"
        SELECT
            p.id,
            p.title,
            p.slug,
            p.body,
            p.featured_image_url,
            p.status,
            p.published_at,
            p.created_at,
            p.updated_at,
            c.id AS category_id,
            c.name AS category_name,
            c.slug AS category_slug
        FROM posts p
        INNER JOIN categories c ON c.id = p.category_id
        ORDER BY p.created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let post_ids = rows
        .iter()
        .map(|row| row.get::<Uuid, _>("id"))
        .collect::<Vec<_>>();
    let tags_by_post = load_tags_for_posts(pool, &post_ids).await?;

    let items = rows
        .into_iter()
        .map(|row| {
            let post_id = row.get::<Uuid, _>("id");
            PostDetails {
                id: post_id,
                title: row.get("title"),
                slug: row.get("slug"),
                body: row.get("body"),
                featured_image_url: row.get("featured_image_url"),
                category: CategorySummary {
                    id: row.get("category_id"),
                    name: row.get("category_name"),
                    slug: row.get("category_slug"),
                },
                status: row.get("status"),
                published_at: row.get("published_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                tags: tags_by_post.get(&post_id).cloned().unwrap_or_default(),
            }
        })
        .collect::<Vec<_>>();

    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*)::BIGINT FROM posts")
        .fetch_one(pool)
        .await?;

    Ok(PaginatedPosts {
        items,
        page: options.page,
        per_page: options.per_page,
        total,
    })
}

pub async fn get_post_by_slug(pool: &PgPool, slug: &str) -> Result<Option<PostDetails>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT
            p.id,
            p.title,
            p.slug,
            p.body,
            p.featured_image_url,
            p.status,
            p.published_at,
            p.created_at,
            p.updated_at,
            c.id AS category_id,
            c.name AS category_name,
            c.slug AS category_slug
        FROM posts p
        INNER JOIN categories c ON c.id = p.category_id
        WHERE p.slug = $1
        "#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let post_id = row.get::<Uuid, _>("id");
    let tags_by_post = load_tags_for_posts(pool, &[post_id]).await?;

    Ok(Some(PostDetails {
        id: post_id,
        title: row.get("title"),
        slug: row.get("slug"),
        body: row.get("body"),
        featured_image_url: row.get("featured_image_url"),
        category: CategorySummary {
            id: row.get("category_id"),
            name: row.get("category_name"),
            slug: row.get("category_slug"),
        },
        status: row.get("status"),
        published_at: row.get("published_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        tags: tags_by_post.get(&post_id).cloned().unwrap_or_default(),
    }))
}

pub async fn update_post(
    pool: &PgPool,
    id: Uuid,
    changes: PostChanges,
) -> Result<Option<PostDetails>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let updated_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        UPDATE posts
        SET
            title = $1,
            slug = $2,
            body = $3,
            featured_image_url = $4,
            category_id = $5,
            status = $6,
            published_at = $7,
            updated_at = NOW()
        WHERE id = $8
        RETURNING id
        "#,
    )
    .bind(changes.title)
    .bind(changes.slug)
    .bind(changes.body)
    .bind(changes.featured_image_url)
    .bind(changes.category_id)
    .bind(changes.status)
    .bind(changes.published_at)
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(updated_id) = updated_id else {
        return Ok(None);
    };

    replace_post_tags(&mut tx, updated_id, &changes.tag_ids).await?;
    tx.commit().await?;

    fetch_post_details_by_id(pool, updated_id).await
}

pub async fn delete_post(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM posts WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

async fn fetch_post_details_by_id(pool: &PgPool, post_id: Uuid) -> Result<Option<PostDetails>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT
            p.id,
            p.title,
            p.slug,
            p.body,
            p.featured_image_url,
            p.status,
            p.published_at,
            p.created_at,
            p.updated_at,
            c.id AS category_id,
            c.name AS category_name,
            c.slug AS category_slug
        FROM posts p
        INNER JOIN categories c ON c.id = p.category_id
        WHERE p.id = $1
        "#,
    )
    .bind(post_id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let tags_by_post = load_tags_for_posts(pool, &[post_id]).await?;

    Ok(Some(PostDetails {
        id: post_id,
        title: row.get("title"),
        slug: row.get("slug"),
        body: row.get("body"),
        featured_image_url: row.get("featured_image_url"),
        category: CategorySummary {
            id: row.get("category_id"),
            name: row.get("category_name"),
            slug: row.get("category_slug"),
        },
        status: row.get("status"),
        published_at: row.get("published_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        tags: tags_by_post.get(&post_id).cloned().unwrap_or_default(),
    }))
}

async fn replace_post_tags(
    tx: &mut Transaction<'_, Postgres>,
    post_id: Uuid,
    tag_ids: &[Uuid],
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM post_tags WHERE post_id = $1")
        .bind(post_id)
        .execute(&mut **tx)
        .await?;

    for tag_id in tag_ids {
        sqlx::query("INSERT INTO post_tags (post_id, tag_id) VALUES ($1, $2)")
            .bind(post_id)
            .bind(*tag_id)
            .execute(&mut **tx)
            .await?;
    }

    Ok(())
}

async fn load_tags_for_posts(
    pool: &PgPool,
    post_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<TagSummary>>, sqlx::Error> {
    if post_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query(
        r#"
        SELECT
            pt.post_id,
            t.id,
            t.name,
            t.slug
        FROM post_tags pt
        INNER JOIN tags t ON t.id = pt.tag_id
        WHERE pt.post_id = ANY($1)
        ORDER BY t.name ASC
        "#,
    )
    .bind(post_ids)
    .fetch_all(pool)
    .await?;

    let mut map = HashMap::<Uuid, Vec<TagSummary>>::new();

    for row in rows {
        let post_id = row.get::<Uuid, _>("post_id");
        let tag = TagSummary {
            id: row.get("id"),
            name: row.get("name"),
            slug: row.get("slug"),
        };

        map.entry(post_id).or_default().push(tag);
    }

    Ok(map)
}
