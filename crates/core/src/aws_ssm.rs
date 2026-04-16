//! AWS SSM Session Manager port forwarding.
//!
//! Spawns `aws ssm start-session ... AWS-StartPortForwardingSessionToRemoteHost`
//! which forwards a random local TCP port to a remote host (e.g. an RDS
//! endpoint) via an EC2 instance running the SSM agent. MySQL then connects
//! to 127.0.0.1:<random_port>.
//!
//! Requirements on the user's machine:
//! - `aws` CLI v2 (logged in / configured)
//! - `session-manager-plugin` (installed separately)

use crate::tunnel::{drain_stderr, find_free_port, wait_for_first_byte, wait_for_port, Tunnel};
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsmConfig {
    /// EC2 instance id that runs the SSM agent (e.g. `i-00e64f3807d1c2061`).
    pub target: String,
    /// Optional AWS region override (otherwise inferred from default profile).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    /// Optional named AWS profile (otherwise the default profile is used).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
}

pub async fn open(ssm: &SsmConfig, target_host: &str, target_port: u16) -> Result<Tunnel> {
    let local_port = find_free_port().context("find free local port for tunnel")?;

    let mut cmd = Command::new("aws");
    cmd.arg("ssm")
        .arg("start-session")
        .arg("--target")
        .arg(&ssm.target)
        .arg("--document-name")
        .arg("AWS-StartPortForwardingSessionToRemoteHost")
        .arg("--parameters")
        .arg(format!(
            r#"{{"host":["{target_host}"],"portNumber":["{target_port}"],"localPortNumber":["{local_port}"]}}"#
        ));
    if let Some(region) = &ssm.region {
        cmd.arg("--region").arg(region);
    }
    if let Some(profile) = &ssm.profile {
        cmd.arg("--profile").arg(profile);
    }

    cmd.stdin(Stdio::null())
        .stdout(Stdio::null()) // SSM logs session info to stdout; we don't need it
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .context("spawn `aws ssm start-session` (is the AWS CLI installed?)")?;

    // 1. Local listener up?
    if let Err(e) = wait_for_port(local_port, Duration::from_secs(20)).await {
        let stderr = drain_stderr(&mut child).await;
        let _ = child.start_kill();
        return Err(anyhow!(
            "AWS SSM tunnel failed: {e}{}",
            if stderr.is_empty() { String::new() } else { format!("\n{}", stderr.trim()) }
        ));
    }

    // 2. Actually delivering bytes? Without this the MySQL pool will time
    //    out on its first connect because the SSM session-manager-plugin
    //    accepts TCP locally well before the AWS↔RDS pipe is fully ready.
    if let Err(e) = wait_for_first_byte(local_port, Duration::from_secs(20)).await {
        let stderr = drain_stderr(&mut child).await;
        let _ = child.start_kill();
        return Err(anyhow!(
            "AWS SSM tunnel handshake failed: {e}{}",
            if stderr.is_empty() { String::new() } else { format!("\n{}", stderr.trim()) }
        ));
    }

    Ok(Tunnel::new(local_port, child))
}
