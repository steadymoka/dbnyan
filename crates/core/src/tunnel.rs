//! Local-port-forwarding tunnel via the system `ssh` binary.
//!
//! For each connection that has SSH config we spawn `ssh -N -L`, which
//! forwards a random local port to the target DB host (from the bastion's
//! perspective). MySQL then connects to 127.0.0.1:<random_port>.
//!
//! Auth: key + agent only. Password auth is interactive in `ssh` (BatchMode
//! disables it), so we surface a friendly error if requested.

use crate::ssh::{SshAuth, SshConfig};
use anyhow::{anyhow, Context, Result};
use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::process::{Child, Command};
use tokio::time::{sleep, timeout, Instant};

pub struct Tunnel {
    pub local_port: u16,
    child: Child,
    /// Process-group id of `child`. Set via `spawn_in_new_process_group` so we
    /// can SIGTERM the whole group — `aws ssm` and `ssh` both fork grandchildren
    /// (session-manager-plugin, auth helpers) that would otherwise be reparented
    /// to launchd/init and hold onto the forwarded port after the direct child
    /// dies.
    pgid: Option<i32>,
}

impl Tunnel {
    pub(crate) fn new(local_port: u16, child: Child) -> Self {
        let pgid = child.id().map(|p| p as i32);
        Self {
            local_port,
            child,
            pgid,
        }
    }
}

impl Drop for Tunnel {
    fn drop(&mut self) {
        #[cfg(unix)]
        if let Some(pgid) = self.pgid {
            // Negative pid ⇒ kill the whole process group.
            // SAFETY: libc::kill with a valid pgid and signal number; no UB.
            unsafe {
                libc::kill(-pgid, libc::SIGTERM);
            }
        }
        // Belt & suspenders: SIGKILL the direct child too, in case the group
        // signal was missed (e.g. pid recycled) or the child ignored SIGTERM.
        let _ = self.child.start_kill();
    }
}

/// Put the spawned child into a new process group (pgid = child pid) on Unix.
/// No-op elsewhere. Call before `spawn`.
pub fn spawn_in_new_process_group(_cmd: &mut Command) {
    #[cfg(unix)]
    {
        _cmd.process_group(0);
    }
}

pub async fn open(ssh: &SshConfig, target_host: &str, target_port: u16) -> Result<Tunnel> {
    let local_port = find_free_port().context("find free local port for tunnel")?;

    let mut cmd = Command::new("ssh");
    cmd.arg("-N")
        .arg("-T")
        .arg("-o").arg("ExitOnForwardFailure=yes")
        .arg("-o").arg("BatchMode=yes")
        .arg("-o").arg("StrictHostKeyChecking=accept-new")
        .arg("-o").arg("ServerAliveInterval=30")
        .arg("-o").arg("ServerAliveCountMax=3")
        .arg("-L").arg(format!("127.0.0.1:{local_port}:{target_host}:{target_port}"))
        .arg("-p").arg(ssh.port.to_string());

    match &ssh.auth {
        SshAuth::Key { key_path, passphrase } => {
            if passphrase.is_some() {
                return Err(anyhow!(
                    "key passphrase is not supported with the system `ssh` BatchMode. \
                     Add the key to ssh-agent (`ssh-add {}`) and switch the connection to 'agent'.",
                    key_path
                ));
            }
            cmd.arg("-i").arg(expand_tilde(key_path));
            cmd.arg("-o").arg("IdentitiesOnly=yes");
        }
        SshAuth::Agent => {
            // BatchMode + no -i; ssh will use the agent.
        }
        SshAuth::Password { .. } => {
            return Err(anyhow!(
                "password SSH auth is not supported in MVP. Use a key or ssh-agent."
            ));
        }
    }

    cmd.arg(format!("{}@{}", ssh.user, ssh.host));
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    spawn_in_new_process_group(&mut cmd);

    let mut child = cmd.spawn().context("spawn `ssh`")?;

    if let Err(e) = wait_for_port(local_port, Duration::from_secs(8)).await {
        let stderr = drain_stderr(&mut child).await;
        let _ = child.start_kill();
        return Err(anyhow!(
            "ssh tunnel failed: {e}{}",
            if stderr.is_empty() { String::new() } else { format!("\n{}", stderr.trim()) }
        ));
    }

    Ok(Tunnel::new(local_port, child))
}

pub(crate) fn find_free_port() -> Result<u16> {
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))?;
    let port = listener.local_addr()?.port();
    drop(listener);
    Ok(port)
}

pub(crate) async fn wait_for_port(port: u16, total: Duration) -> Result<()> {
    let start = Instant::now();
    loop {
        if TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
            return Ok(());
        }
        if start.elapsed() >= total {
            return Err(anyhow!("local forward port {port} did not come up within {total:?}"));
        }
        sleep(Duration::from_millis(100)).await;
    }
}

/// Stronger readiness check: actually open a TCP connection to `port` and wait
/// for the remote to send at least one byte (MySQL greets first). Useful for
/// SSM where the local listener can be up before the SSM ↔ RDS pipe is fully
/// stitched — connecting too early hangs sqlx.
pub(crate) async fn wait_for_first_byte(port: u16, total: Duration) -> Result<()> {
    let start = Instant::now();
    while start.elapsed() < total {
        if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)).await {
            let mut buf = [0u8; 1];
            match timeout(Duration::from_millis(2500), stream.read(&mut buf)).await {
                Ok(Ok(n)) if n > 0 => return Ok(()),
                _ => {} // listener accepted but no greeting yet — close & retry
            }
        }
        sleep(Duration::from_millis(500)).await;
    }
    Err(anyhow!(
        "tunnel port {port} did not deliver any data within {total:?} \
         (forwarder accepts but the remote isn't talking yet)"
    ))
}

pub async fn drain_stderr(child: &mut Child) -> String {
    let Some(mut stderr) = child.stderr.take() else {
        return String::new();
    };
    let mut buf = String::new();
    let _ = timeout(Duration::from_millis(300), stderr.read_to_string(&mut buf)).await;
    buf
}

fn expand_tilde(p: &str) -> String {
    if let Some(rest) = p.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest).to_string_lossy().into_owned();
        }
    }
    p.to_string()
}
