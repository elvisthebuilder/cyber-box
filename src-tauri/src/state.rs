use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc::UnboundedSender, Mutex};

use cyberbox_core::config::Config;
use cyberbox_core::docker::DockerClient;
use cyberbox_core::registry::Registry;

#[derive(serde::Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "kind", content = "detail")]
pub enum InstallStatus {
    Unknown,
    Installed,
    NotInstalled,
    Installing,
    Failed(String),
}

pub struct PtySession {
    pub input: UnboundedSender<Vec<u8>>,
    pub exec_id: String,
}

pub struct AppState {
    pub config: Config,
    pub docker: Arc<DockerClient>,
    pub registry: Registry,
    pub install_status: Mutex<HashMap<String, InstallStatus>>,
    pub pty_sessions: Mutex<HashMap<String, PtySession>>,
}

impl AppState {
    pub fn new(config: Config, docker: Arc<DockerClient>, registry: Registry) -> Self {
        let install_status = registry
            .tools
            .iter()
            .map(|t| (t.name.clone(), InstallStatus::Unknown))
            .collect();
        Self {
            config,
            docker,
            registry,
            install_status: Mutex::new(install_status),
            pty_sessions: Mutex::new(HashMap::new()),
        }
    }
}
