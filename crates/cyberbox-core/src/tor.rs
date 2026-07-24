use anyhow::Result;

use crate::docker::DockerClient;

pub async fn set_enabled(docker: &DockerClient, enabled: bool) -> Result<()> {
    let action = if enabled { "start" } else { "stop" };
    docker
        .exec_oneshot(format!("supervisorctl {action} tor"))
        .await?;
    Ok(())
}

pub async fn is_running(docker: &DockerClient) -> bool {
    match docker.exec_oneshot("supervisorctl status tor".to_string()).await {
        Ok(out) => out.contains("RUNNING"),
        Err(_) => false,
    }
}

/// Wraps a command with torsocks if Tor is enabled and the tool supports it.
pub fn maybe_wrap(cmd: String, tor_enabled: bool, tor_wrappable: bool) -> (String, Option<String>) {
    if tor_enabled && tor_wrappable {
        (format!("torsocks {cmd}"), None)
    } else if tor_enabled && !tor_wrappable {
        (
            cmd,
            Some("Tor is ON but this tool cannot be wrapped (needs raw sockets) — traffic will NOT be anonymized.".to_string()),
        )
    } else {
        (cmd, None)
    }
}
