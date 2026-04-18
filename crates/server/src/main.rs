mod chat;
mod connections;
mod error;
mod runtime;

use anyhow::Result;
use axum::{extract::State, routing::get, Json, Router};
use dbnyan_core::session::SessionManager;
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::path::Path;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub sessions: SessionManager,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn,tower_http=info".into()),
        )
        .init();

    let pool = dbnyan_core::open_app_db().await?;
    let state = AppState {
        pool,
        sessions: SessionManager::new(),
    };

    let shutdown_sessions = state.sessions.clone();
    let api = Router::new()
        .route("/health", get(health))
        .merge(connections::router())
        .merge(runtime::router())
        .merge(chat::router())
        .with_state(state);

    let web_build = Path::new("web/build");
    let static_layer = if web_build.exists() {
        let fallback = ServeFile::new(web_build.join("index.html"));
        Some(ServeDir::new(web_build).not_found_service(fallback))
    } else {
        tracing::warn!(
            "web/build/ not found — API only. Run `bun run build` in web/ or use the dev server."
        );
        None
    };

    let mut app = Router::new().nest("/api", api);
    if let Some(svc) = static_layer {
        app = app.fallback_service(svc);
    }
    let app = app.layer(TraceLayer::new_for_http());

    let port: u16 = std::env::var("DBNYAN_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3939);
    let addr: SocketAddr = ([127, 0, 0, 1], port).into();
    tracing::info!(%addr, "dbnyan listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // Graceful shutdown returned → no more in-flight requests hold Arc<Session>.
    // Clearing the registry drops every Tunnel, which kills its process group.
    shutdown_sessions.shutdown_all().await;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        if let Ok(mut sig) = signal(SignalKind::terminate()) {
            sig.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("received Ctrl-C, shutting down"),
        _ = terminate => tracing::info!("received SIGTERM, shutting down"),
    }
}

async fn health(State(state): State<AppState>) -> Json<Value> {
    let db_ok = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await
        .is_ok();
    Json(json!({
        "status": if db_ok { "ok" } else { "degraded" },
        "app_db": db_ok,
    }))
}
