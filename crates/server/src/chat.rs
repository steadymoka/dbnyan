//! Chat endpoint that spawns the local `claude` CLI as a subprocess.
//!
//! Uses Claude Code subscription auth (inherited from `claude login`). We
//! explicitly drop `ANTHROPIC_API_KEY` from the spawned env so the subprocess
//! never falls back to API billing.
//!
//! First turn (no `session_id`): we prepend a system context with the
//! connection name and the table list of the active database, then the user
//! message. Subsequent turns pass `--resume <session_id>` so claude restores
//! the full conversation state — we only send the new user message.

use crate::error::{AppError, AppResult};
use crate::AppState;
use anyhow::anyhow;
use axum::extract::{Path, State};
use axum::routing::post;
use axum::{Json, Router};
use dbnyan_core::{connection, mysql};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::process::Command;

pub fn router() -> Router<AppState> {
    Router::new().route("/connections/{id}/chat", post(chat))
}

#[derive(Deserialize)]
struct ChatBody {
    message: String,
    /// `None` → start a new session (system context will be prepended).
    /// `Some(sid)` → resume; only `message` is sent to claude.
    session_id: Option<String>,
    /// Currently-selected database in the UI; used to scope schema context.
    database: Option<String>,
}

#[derive(Serialize)]
struct ChatResponse {
    session_id: Option<String>,
    text: String,
    is_error: bool,
    duration_ms: Option<u64>,
}

async fn chat(
    State(state): State<AppState>,
    Path(connection_id): Path<String>,
    Json(body): Json<ChatBody>,
) -> AppResult<Json<ChatResponse>> {
    let conn = connection::get(&state.pool, &connection_id)
        .await?
        .ok_or_else(|| AppError::not_found("connection not found"))?;

    let prompt = if body.session_id.is_none() {
        build_initial_prompt(&state, &conn, body.database.as_deref(), &body.message).await
    } else {
        body.message.clone()
    };

    let mut cmd = Command::new("claude");
    cmd.arg("-p")
        .arg(&prompt)
        .arg("--output-format")
        .arg("json");
    if let Some(sid) = &body.session_id {
        cmd.arg("--resume").arg(sid);
    }
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("ANTHROPIC_AUTH_TOKEN");

    let output = cmd
        .output()
        .await
        .map_err(|e| AppError::from(anyhow!("spawn claude: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::from(anyhow!(
            "claude exited {}: {}",
            output.status,
            stderr.trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).map_err(|e| {
        AppError::from(anyhow!(
            "parse claude json failed: {e}\nraw output:\n{stdout}"
        ))
    })?;

    let session_id = parsed
        .get("session_id")
        .and_then(|v| v.as_str())
        .map(String::from);
    let text = parsed
        .get("result")
        .and_then(|v| v.as_str())
        .unwrap_or("(no response)")
        .to_string();
    let is_error = parsed
        .get("is_error")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let duration_ms = parsed.get("duration_ms").and_then(|v| v.as_u64());

    Ok(Json(ChatResponse {
        session_id,
        text,
        is_error,
        duration_ms,
    }))
}

async fn build_initial_prompt(
    state: &AppState,
    conn: &dbnyan_core::connection::Connection,
    selected_db: Option<&str>,
    user_message: &str,
) -> String {
    let mut sys = format!(
        "You are a SQL assistant for the MySQL connection \"{}\" \
         ({}@{}:{}).\n\
         Help the user write SQL queries. When you provide SQL, format it in \
         ```sql code blocks so the user can run it directly. Be concise.\n",
        conn.name, conn.username, conn.host, conn.port
    );

    let db = selected_db.or(conn.database.as_deref());
    if let Some(db) = db {
        sys.push_str(&format!("\nCurrent database: `{db}`\n"));
        if let Ok(session) = state.sessions.get_or_open(conn).await {
            if let Ok(tables) = mysql::list_tables(&session.pool, db).await {
                if !tables.is_empty() {
                    sys.push_str(&format!("\nTables in `{db}`:\n"));
                    for t in &tables {
                        sys.push_str(&format!("- {}\n", t.name));
                    }
                }
            }
        }
    }

    format!("{sys}\n---\n\n{user_message}")
}
