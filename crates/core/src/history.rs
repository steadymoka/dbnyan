//! Per-connection query history persisted in the app SQLite db.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct HistoryEntry {
    pub id: String,
    pub connection_id: String,
    pub database_name: Option<String>,
    pub sql: String,
    pub success: bool,
    pub error: Option<String>,
    pub rows_affected: Option<i64>,
    pub rows_returned: Option<i64>,
    pub duration_ms: i64,
    pub executed_at: DateTime<Utc>,
}

#[allow(clippy::too_many_arguments)]
pub async fn record(
    pool: &SqlitePool,
    connection_id: &str,
    database: Option<&str>,
    sql: &str,
    success: bool,
    error: Option<&str>,
    rows_affected: Option<i64>,
    rows_returned: Option<i64>,
    duration_ms: i64,
) -> Result<HistoryEntry> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO query_history \
         (id, connection_id, database_name, sql, success, error, \
          rows_affected, rows_returned, duration_ms, executed_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(connection_id)
    .bind(database)
    .bind(sql)
    .bind(success)
    .bind(error)
    .bind(rows_affected)
    .bind(rows_returned)
    .bind(duration_ms)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(HistoryEntry {
        id,
        connection_id: connection_id.to_string(),
        database_name: database.map(str::to_string),
        sql: sql.to_string(),
        success,
        error: error.map(str::to_string),
        rows_affected,
        rows_returned,
        duration_ms,
        executed_at: now,
    })
}

pub async fn list(pool: &SqlitePool, connection_id: &str, limit: u32) -> Result<Vec<HistoryEntry>> {
    let entries: Vec<HistoryEntry> = sqlx::query_as(
        "SELECT id, connection_id, database_name, sql, success, error, \
                rows_affected, rows_returned, duration_ms, executed_at \
         FROM query_history WHERE connection_id = ? \
         ORDER BY executed_at DESC LIMIT ?",
    )
    .bind(connection_id)
    .bind(i64::from(limit))
    .fetch_all(pool)
    .await?;
    Ok(entries)
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<bool> {
    let res = sqlx::query("DELETE FROM query_history WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

pub async fn clear(pool: &SqlitePool, connection_id: &str) -> Result<u64> {
    let res = sqlx::query("DELETE FROM query_history WHERE connection_id = ?")
        .bind(connection_id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected())
}
