//! Per-connection saved/named queries.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Favorite {
    pub id: String,
    pub connection_id: String,
    pub name: String,
    pub sql: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn list(pool: &SqlitePool, connection_id: &str) -> Result<Vec<Favorite>> {
    let rows: Vec<Favorite> = sqlx::query_as(
        "SELECT id, connection_id, name, sql, created_at, updated_at \
         FROM query_favorites WHERE connection_id = ? \
         ORDER BY name COLLATE NOCASE",
    )
    .bind(connection_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn create(
    pool: &SqlitePool,
    connection_id: &str,
    name: &str,
    sql: &str,
) -> Result<Favorite> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO query_favorites (id, connection_id, name, sql, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(connection_id)
    .bind(name)
    .bind(sql)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(Favorite {
        id,
        connection_id: connection_id.to_string(),
        name: name.to_string(),
        sql: sql.to_string(),
        created_at: now,
        updated_at: now,
    })
}

pub async fn update(
    pool: &SqlitePool,
    id: &str,
    name: Option<&str>,
    sql: Option<&str>,
) -> Result<Option<Favorite>> {
    let now = Utc::now();
    // We update individual columns only when provided so callers can change
    // just the name (rename) without overwriting the SQL.
    let res = sqlx::query(
        "UPDATE query_favorites SET \
         name = COALESCE(?, name), sql = COALESCE(?, sql), updated_at = ? \
         WHERE id = ?",
    )
    .bind(name)
    .bind(sql)
    .bind(now)
    .bind(id)
    .execute(pool)
    .await?;

    if res.rows_affected() == 0 {
        return Ok(None);
    }

    let row: Option<Favorite> = sqlx::query_as(
        "SELECT id, connection_id, name, sql, created_at, updated_at \
         FROM query_favorites WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<bool> {
    let res = sqlx::query("DELETE FROM query_favorites WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}
