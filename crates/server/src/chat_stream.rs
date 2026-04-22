//! Streaming variant of `/chat`: pipes the `claude -p` CLI's stream-json
//! output to the frontend as Server-Sent Events.
//!
//! The CLI emits NDJSON; we map the interesting lines to three SSE event
//! kinds — `session`, `text_delta`, `done`, plus `error` on failure.
//! Everything else (rate_limit, message_start/stop, content_block_start/stop,
//! accumulated `type=assistant` snapshots) is ignored; deltas come from
//! `stream_event` / `content_block_delta`, which is what `--include-partial-messages`
//! enables.
//!
//! Cancellation is handled by RAII: the spawned child lives inside a
//! [`ChildGuard`](dbnyan_core::ChildGuard) owned by the stream future. When
//! the client disconnects, axum drops the stream → the guard is dropped →
//! SIGTERM hits the whole process group (so `claude` and any grandchildren
//! die together).

use crate::chat::{build_prompt, claude_bin, configure_claude_cmd, ChatBody};
use crate::error::{AppError, AppResult};
use crate::AppState;
use anyhow::anyhow;
use axum::extract::{Path, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use dbnyan_core::{connection, ChildGuard};
use futures::Stream;
use serde_json::{json, Value};
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

pub async fn chat_stream(
    State(state): State<AppState>,
    Path(connection_id): Path<String>,
    Json(body): Json<ChatBody>,
) -> AppResult<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    let conn = connection::get(&state.pool, &connection_id)
        .await?
        .ok_or_else(|| AppError::not_found("connection not found"))?;

    let prompt = build_prompt(&state, &conn, &body).await;

    let mut cmd = Command::new(claude_bin());
    cmd.arg("-p")
        .arg(&prompt)
        .arg("--output-format")
        .arg("stream-json")
        .arg("--verbose")
        .arg("--include-partial-messages");
    if let Some(sid) = &body.session_id {
        cmd.arg("--resume").arg(sid);
    }
    configure_claude_cmd(&mut cmd);
    dbnyan_core::spawn_in_new_process_group(&mut cmd);

    let child = cmd
        .spawn()
        .map_err(|e| AppError::from(anyhow!("spawn claude: {e}")))?;
    let mut guard = ChildGuard::new(child);
    let stdout = guard
        .take_stdout()
        .ok_or_else(|| AppError::from(anyhow!("claude child has no stdout")))?;
    let stderr = guard
        .take_stderr()
        .ok_or_else(|| AppError::from(anyhow!("claude child has no stderr")))?;

    // Drain stderr in parallel so the pipe never fills and blocks the child.
    // We only surface it if the child dies without producing a `result` line.
    let stderr_buf = Arc::new(Mutex::new(String::new()));
    {
        let buf = stderr_buf.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if let Ok(mut guard) = buf.lock() {
                    if !guard.is_empty() {
                        guard.push('\n');
                    }
                    guard.push_str(&line);
                }
            }
        });
    }

    let stream = async_stream::stream! {
        let _guard = guard;
        let mut lines = BufReader::new(stdout).lines();
        let mut saw_result = false;

        loop {
            match lines.next_line().await {
                Ok(Some(raw)) => {
                    let v: Value = match serde_json::from_str(&raw) {
                        Ok(v) => v,
                        Err(e) => {
                            tracing::warn!(line = %raw, error = %e, "chat_stream: parse");
                            yield Ok(error_event(&format!("parse: {e}")));
                            break;
                        }
                    };
                    for ev in map_stream_json(&v) {
                        yield Ok(ev);
                    }
                    if v.get("type").and_then(|t| t.as_str()) == Some("result") {
                        saw_result = true;
                        break;
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    yield Ok(error_event(&format!("read: {e}")));
                    break;
                }
            }
        }

        if !saw_result {
            let tail = stderr_buf.lock().ok().map(|g| g.clone()).unwrap_or_default();
            let msg = if tail.trim().is_empty() {
                "claude exited without producing a result".to_string()
            } else {
                tail.trim().to_string()
            };
            yield Ok(error_event(&msg));
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

/// Translate a single stream-json line into zero or more SSE events.
fn map_stream_json(v: &Value) -> Vec<Event> {
    let kind = v.get("type").and_then(|t| t.as_str()).unwrap_or("");

    match kind {
        "system" => {
            if v.get("subtype").and_then(|s| s.as_str()) == Some("init") {
                if let Some(sid) = v.get("session_id").and_then(|s| s.as_str()) {
                    return vec![named_event(
                        "session",
                        &json!({ "session_id": sid }),
                    )];
                }
            }
            Vec::new()
        }
        "stream_event" => {
            let evt = match v.get("event") {
                Some(e) => e,
                None => return Vec::new(),
            };
            if evt.get("type").and_then(|t| t.as_str()) == Some("content_block_delta") {
                if let Some(delta) = evt.get("delta") {
                    if delta.get("type").and_then(|t| t.as_str()) == Some("text_delta") {
                        if let Some(text) = delta.get("text").and_then(|t| t.as_str()) {
                            if !text.is_empty() {
                                return vec![named_event(
                                    "text_delta",
                                    &json!({ "delta": text }),
                                )];
                            }
                        }
                    }
                }
            }
            Vec::new()
        }
        "result" => {
            let is_error = v
                .get("is_error")
                .and_then(|b| b.as_bool())
                .unwrap_or(false);
            if is_error {
                let msg = v
                    .get("result")
                    .and_then(|r| r.as_str())
                    .or_else(|| v.get("subtype").and_then(|s| s.as_str()))
                    .unwrap_or("unknown error");
                vec![error_event(msg)]
            } else {
                let duration = v.get("duration_ms").and_then(|d| d.as_u64());
                let payload = match duration {
                    Some(ms) => json!({ "duration_ms": ms }),
                    None => json!({}),
                };
                vec![named_event("done", &payload)]
            }
        }
        _ => Vec::new(),
    }
}

fn named_event(name: &str, data: &Value) -> Event {
    Event::default()
        .event(name)
        .json_data(data)
        .unwrap_or_else(|_| Event::default().event(name).data(data.to_string()))
}

fn error_event(message: &str) -> Event {
    named_event("error", &json!({ "message": message }))
}
