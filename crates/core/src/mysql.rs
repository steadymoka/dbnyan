//! Read-side MySQL helpers: list databases/tables, describe schema, preview rows.

use anyhow::Result;
use serde::Serialize;
use sqlx::mysql::{MySqlPool, MySqlRow};
use sqlx::{Column, Row, TypeInfo, ValueRef};

const SYSTEM_SCHEMAS: &[&str] = &[
    "information_schema",
    "performance_schema",
    "mysql",
    "sys",
];

pub async fn list_databases(pool: &MySqlPool) -> Result<Vec<String>> {
    let rows = sqlx::query("SHOW DATABASES").fetch_all(pool).await?;
    Ok(rows
        .into_iter()
        .map(|r| read_text(&r, 0))
        .filter(|d| !SYSTEM_SCHEMAS.contains(&d.as_str()))
        .collect())
}

#[derive(Debug, Serialize)]
pub struct TableInfo {
    pub name: String,
    pub kind: String,
}

pub async fn list_tables(pool: &MySqlPool, db: &str) -> Result<Vec<TableInfo>> {
    let sql = format!("SHOW FULL TABLES FROM {}", quote_ident(db));
    let rows = sqlx::query(&sql).fetch_all(pool).await?;
    Ok(rows
        .into_iter()
        .map(|r| TableInfo {
            name: read_text(&r, 0),
            kind: read_text(&r, 1),
        })
        .collect())
}

#[derive(Debug, Serialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default: Option<String>,
    pub key: Option<String>,
    pub extra: Option<String>,
}

pub async fn describe_table(pool: &MySqlPool, db: &str, table: &str) -> Result<Vec<ColumnInfo>> {
    // SHOW FULL COLUMNS FROM `db`.`table`
    // Columns: Field, Type, Collation, Null, Key, Default, Extra, Privileges, Comment
    let sql = format!(
        "SHOW FULL COLUMNS FROM {}.{}",
        quote_ident(db),
        quote_ident(table)
    );
    let rows = sqlx::query(&sql).fetch_all(pool).await?;
    Ok(rows
        .into_iter()
        .map(|r| {
            let nullable = read_text(&r, 3);
            let key = read_text(&r, 4);
            let extra = read_text(&r, 6);
            ColumnInfo {
                name: read_text(&r, 0),
                data_type: read_text(&r, 1),
                nullable: nullable.eq_ignore_ascii_case("YES"),
                default: read_text_opt(&r, 5),
                key: (!key.is_empty()).then_some(key),
                extra: (!extra.is_empty()).then_some(extra),
            }
        })
        .collect())
}

#[derive(Debug, Serialize)]
pub struct RowSet {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub limit: u32,
    pub returned: usize,
}

pub async fn preview_rows(
    pool: &MySqlPool,
    db: &str,
    table: &str,
    limit: u32,
) -> Result<RowSet> {
    let limit = limit.clamp(1, 1000);
    let sql = format!(
        "SELECT * FROM {}.{} LIMIT {}",
        quote_ident(db),
        quote_ident(table),
        limit
    );
    let rows = sqlx::query(&sql).fetch_all(pool).await?;

    let columns: Vec<String> = rows
        .first()
        .map(|r| r.columns().iter().map(|c| c.name().to_string()).collect())
        .unwrap_or_default();

    let returned = rows.len();
    let json_rows: Vec<Vec<serde_json::Value>> = rows
        .iter()
        .map(|row| (0..row.len()).map(|i| value_to_json(row, i)).collect())
        .collect();

    Ok(RowSet {
        columns,
        rows: json_rows,
        limit,
        returned,
    })
}

fn quote_ident(s: &str) -> String {
    format!("`{}`", s.replace('`', "``"))
}

/// Read a textual column tolerantly: many MySQL metadata columns are returned
/// with the binary flag set even though the bytes are valid UTF-8. Try `String`
/// first, fall back to `Vec<u8>` + lossy UTF-8 conversion.
fn read_text(row: &MySqlRow, idx: usize) -> String {
    if let Ok(s) = row.try_get::<String, _>(idx) {
        return s;
    }
    if let Ok(b) = row.try_get::<Vec<u8>, _>(idx) {
        return String::from_utf8_lossy(&b).into_owned();
    }
    String::new()
}

fn read_text_opt(row: &MySqlRow, idx: usize) -> Option<String> {
    if let Ok(raw) = row.try_get_raw(idx) {
        if raw.is_null() {
            return None;
        }
    }
    Some(read_text(row, idx))
}

/// Best-effort conversion of a MySQL cell into a JSON value.
pub(crate) fn value_to_json(row: &MySqlRow, idx: usize) -> serde_json::Value {
    let raw = match row.try_get_raw(idx) {
        Ok(r) => r,
        Err(_) => return serde_json::Value::Null,
    };
    if raw.is_null() {
        return serde_json::Value::Null;
    }
    let type_name = raw.type_info().name().to_uppercase();

    let try_str = || -> serde_json::Value {
        if let Ok(s) = row.try_get::<String, _>(idx) {
            return serde_json::Value::String(s);
        }
        if let Ok(b) = row.try_get::<Vec<u8>, _>(idx) {
            return serde_json::Value::String(String::from_utf8_lossy(&b).into_owned());
        }
        serde_json::Value::String(format!("<{type_name}>"))
    };

    match type_name.as_str() {
        "TINYINT" => row
            .try_get::<i16, _>(idx)
            .map(|v| serde_json::json!(v))
            .unwrap_or(serde_json::Value::Null),
        "TINYINT UNSIGNED" => row
            .try_get::<u16, _>(idx)
            .map(|v| serde_json::json!(v))
            .unwrap_or(serde_json::Value::Null),
        "SMALLINT" => row
            .try_get::<i16, _>(idx)
            .map(|v| serde_json::json!(v))
            .unwrap_or(serde_json::Value::Null),
        "SMALLINT UNSIGNED" => row
            .try_get::<u16, _>(idx)
            .map(|v| serde_json::json!(v))
            .unwrap_or(serde_json::Value::Null),
        "INT" | "MEDIUMINT" => row
            .try_get::<i32, _>(idx)
            .map(|v| serde_json::json!(v))
            .unwrap_or(serde_json::Value::Null),
        "MEDIUMINT UNSIGNED" => row
            .try_get::<u32, _>(idx)
            .map(|v| serde_json::json!(v))
            .unwrap_or(serde_json::Value::Null),
        "INT UNSIGNED" => row
            .try_get::<u32, _>(idx)
            .map(|v| serde_json::json!(v))
            .unwrap_or(serde_json::Value::Null),
        "BIGINT" => row
            .try_get::<i64, _>(idx)
            .map(|v| serde_json::json!(v))
            .unwrap_or(serde_json::Value::Null),
        "BIGINT UNSIGNED" => row
            .try_get::<u64, _>(idx)
            .map(|v| serde_json::json!(v))
            .unwrap_or(serde_json::Value::Null),
        "FLOAT" => row
            .try_get::<f32, _>(idx)
            .map(|v| serde_json::json!(v))
            .unwrap_or(serde_json::Value::Null),
        "DOUBLE" => row
            .try_get::<f64, _>(idx)
            .map(|v| serde_json::json!(v))
            .unwrap_or(serde_json::Value::Null),
        "DECIMAL" | "NEWDECIMAL" => try_str(),
        "VARCHAR" | "CHAR" | "TEXT" | "TINYTEXT" | "MEDIUMTEXT" | "LONGTEXT" | "ENUM" | "SET" => {
            try_str()
        }
        "DATE" => row
            .try_get::<chrono::NaiveDate, _>(idx)
            .map(|v| serde_json::json!(v.to_string()))
            .unwrap_or_else(|_| try_str()),
        "TIME" => row
            .try_get::<chrono::NaiveTime, _>(idx)
            .map(|v| serde_json::json!(v.to_string()))
            .unwrap_or_else(|_| try_str()),
        "DATETIME" => row
            .try_get::<chrono::NaiveDateTime, _>(idx)
            .map(|v| serde_json::json!(v.format("%Y-%m-%d %H:%M:%S%.f").to_string()))
            .unwrap_or_else(|_| try_str()),
        "TIMESTAMP" => row
            .try_get::<chrono::DateTime<chrono::Utc>, _>(idx)
            .map(|v| serde_json::json!(v.to_rfc3339()))
            .unwrap_or_else(|_| {
                row.try_get::<chrono::NaiveDateTime, _>(idx)
                    .map(|v| serde_json::json!(v.format("%Y-%m-%d %H:%M:%S%.f").to_string()))
                    .unwrap_or_else(|_| try_str())
            }),
        "YEAR" => row
            .try_get::<i16, _>(idx)
            .map(|v| serde_json::json!(v))
            .unwrap_or(serde_json::Value::Null),
        "JSON" => row
            .try_get::<serde_json::Value, _>(idx)
            .unwrap_or(serde_json::Value::Null),
        "BLOB" | "TINYBLOB" | "MEDIUMBLOB" | "LONGBLOB" | "BINARY" | "VARBINARY" => row
            .try_get::<Vec<u8>, _>(idx)
            .map(|v| serde_json::json!(format!("<binary, {} bytes>", v.len())))
            .unwrap_or(serde_json::Value::Null),
        _ => try_str(),
    }
}
