//! dbnyan core: app-state storage and DB driver primitives.

pub mod aws_ssm;
pub mod connection;
pub mod history;
pub mod mysql;
pub mod query;
pub mod session;
pub mod ssh;
pub mod tunnel;

use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;
use std::str::FromStr;
use tracing::info;

/// Resolve the on-disk location of dbnyan's own state database.
///
/// Default: `<platform data_dir>/dbnyan/app.db` (macOS:
/// `~/Library/Application Support/dbnyan/app.db`).
/// Override with env `DBNYAN_DATA_DIR` (useful during development).
pub fn app_db_path() -> Result<PathBuf> {
    let dir = if let Ok(override_dir) = std::env::var("DBNYAN_DATA_DIR") {
        PathBuf::from(override_dir)
    } else {
        dirs::data_dir()
            .context("no platform data_dir available")?
            .join("dbnyan")
    };
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("create data dir: {}", dir.display()))?;
    Ok(dir.join("app.db"))
}

/// Open (or create) the app-state SQLite pool and run migrations.
pub async fn open_app_db() -> Result<SqlitePool> {
    let path = app_db_path()?;
    info!(db_path = %path.display(), "opening app state db");

    let opts = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await
        .context("connect to app state db")?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("run migrations")?;

    Ok(pool)
}
