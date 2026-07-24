use tokio::sync::mpsc::{self, UnboundedReceiver};

use crate::docker::{DockerClient, OutputEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    Running,
    Finished { exit_code: Option<i64> },
    Failed,
}

pub struct ToolSession {
    pub label: String,
    pub lines: Vec<String>,
    pub status: SessionStatus,
    rx: UnboundedReceiver<OutputEvent>,
}

impl ToolSession {
    pub fn launch(docker: &DockerClient, label: String, cmd: String) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        docker.exec_stream(cmd, tx);
        Self {
            label,
            lines: Vec::new(),
            status: SessionStatus::Running,
            rx,
        }
    }

    /// Drains any pending output events without blocking. Call each UI tick.
    pub fn poll(&mut self) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                OutputEvent::Line(l) => self.lines.push(l),
                OutputEvent::Done { exit_code } => {
                    self.status = SessionStatus::Finished { exit_code };
                    self.lines.push(format!(
                        "--- finished (exit code: {}) ---",
                        exit_code.map(|c| c.to_string()).unwrap_or_else(|| "?".into())
                    ));
                }
                OutputEvent::Error(e) => {
                    self.status = SessionStatus::Failed;
                    self.lines.push(format!("--- error: {e} ---"));
                }
            }
        }
    }
}
