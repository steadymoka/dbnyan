//! Active MySQL sessions, keyed by connection id.
//!
//! A `Session` owns a sqlx MySQL pool and (optionally) an SSH tunnel that
//! shuts down when the session is dropped. The `SessionManager` keeps a
//! shared registry; the first request for a connection lazily opens it.

use crate::aws_ssm;
use crate::connection::Connection;
use crate::tunnel::{self, Tunnel};
use anyhow::{Context, Result};
use sqlx::mysql::{MySqlConnectOptions, MySqlPool, MySqlPoolOptions};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};

pub struct Session {
    pub pool: MySqlPool,
    /// Holds the tunnel alive; dropped when the session is dropped.
    _tunnel: Option<Tunnel>,
}

#[derive(Clone, Default)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Arc<Session>>>>,
    open_lock: Arc<Mutex<()>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the active session, opening one if necessary. Concurrent first
    /// callers are serialized; subsequent reads are cheap.
    pub async fn get_or_open(&self, conn: &Connection) -> Result<Arc<Session>> {
        if let Some(s) = self.sessions.read().await.get(&conn.id).cloned() {
            return Ok(s);
        }

        let _guard = self.open_lock.lock().await;

        if let Some(s) = self.sessions.read().await.get(&conn.id).cloned() {
            return Ok(s);
        }

        let session = open(conn)
            .await
            .with_context(|| format!("open session for `{}`", conn.name))?;
        let session = Arc::new(session);
        self.sessions
            .write()
            .await
            .insert(conn.id.clone(), session.clone());
        Ok(session)
    }

    pub async fn close(&self, connection_id: &str) -> bool {
        self.sessions.write().await.remove(connection_id).is_some()
    }

    pub async fn is_active(&self, connection_id: &str) -> bool {
        self.sessions.read().await.contains_key(connection_id)
    }

    /// Drop every active session. Each `Session` drops its `Tunnel`, which
    /// SIGTERMs the aws/ssh process group — ensuring `session-manager-plugin`
    /// and similar grandchildren don't linger past server shutdown.
    pub async fn shutdown_all(&self) {
        let mut map = self.sessions.write().await;
        let n = map.len();
        map.clear();
        if n > 0 {
            tracing::info!("closed {n} db session(s) on shutdown");
        }
    }
}

async fn open(conn: &Connection) -> Result<Session> {
    let (host, port, tunnel) = if let Some(ssh) = &conn.ssh {
        let t = tunnel::open(ssh, &conn.host, conn.port).await?;
        ("127.0.0.1".to_string(), t.local_port, Some(t))
    } else if let Some(ssm) = &conn.aws_ssm {
        let t = aws_ssm::open(ssm, &conn.host, conn.port).await?;
        ("127.0.0.1".to_string(), t.local_port, Some(t))
    } else {
        (conn.host.clone(), conn.port, None)
    };

    let mut opts = MySqlConnectOptions::new()
        .host(&host)
        .port(port)
        .username(&conn.username);
    if let Some(pw) = &conn.password {
        opts = opts.password(pw);
    }
    if let Some(db) = &conn.database {
        opts = opts.database(db);
    }

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .connect_with(opts)
        .await
        .context("connect to mysql")?;

    Ok(Session {
        pool,
        _tunnel: tunnel,
    })
}
