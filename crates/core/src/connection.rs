use crate::ssh::SshConfig;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub database: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssh: Option<SshConfig>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConnectionInput {
    pub name: String,
    pub host: String,
    #[serde(default = "default_mysql_port")]
    pub port: u16,
    pub username: String,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub database: Option<String>,
    #[serde(default)]
    pub folder: Option<String>,
    #[serde(default)]
    pub ssh: Option<SshConfig>,
}

fn default_mysql_port() -> u16 {
    3306
}

#[derive(sqlx::FromRow)]
struct Row {
    id: String,
    name: String,
    driver: String,
    host: String,
    port: i64,
    username: String,
    password: Option<String>,
    database_name: Option<String>,
    folder: Option<String>,
    ssh_json: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Row {
    fn into_connection(self) -> Result<Connection> {
        let ssh = match self.ssh_json {
            Some(s) => Some(serde_json::from_str(&s).context("parse ssh_json")?),
            None => None,
        };
        Ok(Connection {
            id: self.id,
            name: self.name,
            driver: self.driver,
            host: self.host,
            port: u16::try_from(self.port).context("port out of range")?,
            username: self.username,
            password: self.password,
            database: self.database_name,
            folder: self.folder,
            ssh,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

const SELECT_COLUMNS: &str = "id, name, driver, host, port, username, password,
    database_name, folder, ssh_json, created_at, updated_at";

pub async fn list(pool: &SqlitePool) -> Result<Vec<Connection>> {
    let sql = format!(
        "SELECT {SELECT_COLUMNS} FROM connections \
         ORDER BY COALESCE(folder, ''), name"
    );
    let rows: Vec<Row> = sqlx::query_as(&sql).fetch_all(pool).await?;
    rows.into_iter().map(Row::into_connection).collect()
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<Connection>> {
    let sql = format!("SELECT {SELECT_COLUMNS} FROM connections WHERE id = ?");
    let row: Option<Row> = sqlx::query_as(&sql).bind(id).fetch_optional(pool).await?;
    row.map(Row::into_connection).transpose()
}

pub async fn create(pool: &SqlitePool, input: ConnectionInput) -> Result<Connection> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let ssh_json = input
        .ssh
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .context("serialize ssh")?;

    sqlx::query(
        "INSERT INTO connections \
         (id, name, driver, host, port, username, password, \
          database_name, folder, ssh_json, created_at, updated_at) \
         VALUES (?, ?, 'mysql', ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.host)
    .bind(i64::from(input.port))
    .bind(&input.username)
    .bind(&input.password)
    .bind(&input.database)
    .bind(&input.folder)
    .bind(&ssh_json)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(Connection {
        id,
        name: input.name,
        driver: "mysql".into(),
        host: input.host,
        port: input.port,
        username: input.username,
        password: input.password,
        database: input.database,
        folder: input.folder,
        ssh: input.ssh,
        created_at: now,
        updated_at: now,
    })
}

pub async fn update(pool: &SqlitePool, id: &str, input: ConnectionInput) -> Result<Option<Connection>> {
    let now = Utc::now();
    let ssh_json = input
        .ssh
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .context("serialize ssh")?;

    let res = sqlx::query(
        "UPDATE connections SET \
         name = ?, host = ?, port = ?, username = ?, password = ?, \
         database_name = ?, folder = ?, ssh_json = ?, updated_at = ? \
         WHERE id = ?",
    )
    .bind(&input.name)
    .bind(&input.host)
    .bind(i64::from(input.port))
    .bind(&input.username)
    .bind(&input.password)
    .bind(&input.database)
    .bind(&input.folder)
    .bind(&ssh_json)
    .bind(now)
    .bind(id)
    .execute(pool)
    .await?;

    if res.rows_affected() == 0 {
        return Ok(None);
    }
    get(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<bool> {
    let res = sqlx::query("DELETE FROM connections WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}
