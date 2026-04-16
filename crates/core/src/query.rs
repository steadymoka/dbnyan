//! Run an arbitrary SQL statement against an active MySQL session.
//!
//! Detects whether the statement returns rows (SELECT-like) or affects rows
//! (INSERT/UPDATE/DELETE/DDL) by inspecting the leading keyword.

use crate::mysql::value_to_json;
use anyhow::{anyhow, Result};
use serde::Serialize;
use sqlx::mysql::MySqlPool;
use sqlx::{Column, Row};

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum QueryResult {
    Rows {
        columns: Vec<String>,
        rows: Vec<Vec<serde_json::Value>>,
        returned: usize,
    },
    Affected {
        rows_affected: u64,
        last_insert_id: u64,
    },
}

pub async fn run(pool: &MySqlPool, sql: &str) -> Result<QueryResult> {
    let trimmed = sql.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("empty query"));
    }

    if is_select_like(trimmed) {
        let rows = sqlx::query(trimmed).fetch_all(pool).await?;
        let columns: Vec<String> = rows
            .first()
            .map(|r| r.columns().iter().map(|c| c.name().to_string()).collect())
            .unwrap_or_default();
        let returned = rows.len();
        let json_rows: Vec<Vec<serde_json::Value>> = rows
            .iter()
            .map(|row| (0..row.len()).map(|i| value_to_json(row, i)).collect())
            .collect();
        Ok(QueryResult::Rows {
            columns,
            rows: json_rows,
            returned,
        })
    } else {
        let res = sqlx::query(trimmed).execute(pool).await?;
        Ok(QueryResult::Affected {
            rows_affected: res.rows_affected(),
            last_insert_id: res.last_insert_id(),
        })
    }
}

/// Returns true if the statement likely produces a row set.
fn is_select_like(sql: &str) -> bool {
    let stripped = strip_leading_comments(sql);
    let first: String = stripped
        .chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .collect();
    matches!(
        first.to_uppercase().as_str(),
        "SELECT" | "SHOW" | "DESCRIBE" | "DESC" | "EXPLAIN" | "WITH" | "VALUES"
    )
}

fn strip_leading_comments(sql: &str) -> &str {
    let mut s = sql.trim_start();
    loop {
        if let Some(rest) = s.strip_prefix("--") {
            let end = rest.find('\n').map(|i| i + 1).unwrap_or(rest.len());
            s = rest[end..].trim_start();
        } else if let Some(rest) = s.strip_prefix("/*") {
            let end = rest.find("*/").map(|i| i + 2).unwrap_or(rest.len());
            s = rest[end..].trim_start();
        } else if let Some(rest) = s.strip_prefix('#') {
            let end = rest.find('\n').map(|i| i + 1).unwrap_or(rest.len());
            s = rest[end..].trim_start();
        } else {
            break;
        }
    }
    s
}
