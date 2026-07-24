use anyhow::{Context, Result};
use bollard::container::{
    Config as ContainerConfig, CreateContainerOptions, StartContainerOptions,
};
use bollard::exec::{CreateExecOptions, ResizeExecOptions, StartExecResults};
use bollard::models::HostConfig;
use bollard::Docker;
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::UnboundedSender;

use crate::config::Config;

#[derive(Debug, Clone)]
pub enum OutputEvent {
    Line(String),
    Done { exit_code: Option<i64> },
    Error(String),
}

pub struct DockerClient {
    docker: Docker,
    container_name: String,
    image_tag: String,
}

impl DockerClient {
    pub fn connect(config: &Config) -> Result<Self> {
        let docker =
            Docker::connect_with_local_defaults().context("failed to connect to Docker daemon")?;
        Ok(Self {
            docker,
            container_name: config.container_name.clone(),
            image_tag: config.image_tag.clone(),
        })
    }

    /// Creates the toolbox container if it doesn't exist, starts it if stopped.
    pub async fn ensure_container(&self) -> Result<()> {
        match self
            .docker
            .inspect_container(&self.container_name, None)
            .await
        {
            Ok(info) => {
                let running = info.state.as_ref().and_then(|s| s.running).unwrap_or(false);
                if !running {
                    self.docker
                        .start_container(
                            &self.container_name,
                            None::<StartContainerOptions<String>>,
                        )
                        .await
                        .context("failed to start existing toolbox container")?;
                }
                Ok(())
            }
            Err(bollard::errors::Error::DockerResponseServerError {
                status_code: 404, ..
            }) => {
                let options = CreateContainerOptions {
                    name: self.container_name.clone(),
                    platform: None,
                };
                let config = ContainerConfig {
                    image: Some(self.image_tag.clone()),
                    tty: Some(true),
                    host_config: Some(HostConfig {
                        // nmap/masscan raw-socket scans and Tor's transparent
                        // proxying need these; Docker drops them by default.
                        cap_add: Some(vec!["NET_RAW".to_string(), "NET_ADMIN".to_string()]),
                        ..Default::default()
                    }),
                    ..Default::default()
                };
                self.docker
                    .create_container(Some(options), config)
                    .await
                    .context("failed to create toolbox container")?;
                self.docker
                    .start_container(&self.container_name, None::<StartContainerOptions<String>>)
                    .await
                    .context("failed to start newly created toolbox container")?;
                Ok(())
            }
            Err(e) => Err(e).context("failed to inspect toolbox container"),
        }
    }

    pub async fn is_running(&self) -> bool {
        self.docker
            .inspect_container(&self.container_name, None)
            .await
            .ok()
            .and_then(|info| info.state.and_then(|s| s.running))
            .unwrap_or(false)
    }

    /// Runs `cmd` inside the toolbox container via `sh -c`, streaming stdout/stderr
    /// line-by-line into `tx` until completion.
    pub fn exec_stream(&self, cmd: String, tx: UnboundedSender<OutputEvent>) {
        let docker = self.docker.clone();
        let container_name = self.container_name.clone();
        tokio::spawn(async move {
            let exec = docker
                .create_exec(
                    &container_name,
                    CreateExecOptions {
                        cmd: Some(vec!["sh".to_string(), "-c".to_string(), cmd]),
                        attach_stdout: Some(true),
                        attach_stderr: Some(true),
                        ..Default::default()
                    },
                )
                .await;

            let exec = match exec {
                Ok(e) => e,
                Err(e) => {
                    let _ = tx.send(OutputEvent::Error(format!("exec create failed: {e}")));
                    return;
                }
            };

            match docker.start_exec(&exec.id, None).await {
                Ok(StartExecResults::Attached { mut output, .. }) => {
                    while let Some(chunk) = output.next().await {
                        match chunk {
                            Ok(log) => {
                                let text = log.to_string();
                                for line in text.lines() {
                                    let _ = tx.send(OutputEvent::Line(line.to_string()));
                                }
                            }
                            Err(e) => {
                                let _ =
                                    tx.send(OutputEvent::Error(format!("exec stream error: {e}")));
                                break;
                            }
                        }
                    }
                }
                Ok(StartExecResults::Detached) => {}
                Err(e) => {
                    let _ = tx.send(OutputEvent::Error(format!("exec start failed: {e}")));
                    return;
                }
            }

            let exit_code = docker
                .inspect_exec(&exec.id)
                .await
                .ok()
                .and_then(|i| i.exit_code);
            let _ = tx.send(OutputEvent::Done { exit_code });
        });
    }

    /// Opens a real interactive shell (tty-attached exec) inside the toolbox
    /// container. Raw output bytes (including ANSI escapes) are forwarded to
    /// `on_output` as they arrive; the returned sender accepts raw keystroke
    /// bytes to write to the shell's stdin. Returns the exec id (needed for
    /// resizing). The shell exits when the input sender is dropped (EOF).
    pub async fn exec_interactive(
        &self,
        cmd: Vec<String>,
        on_output: UnboundedSender<Vec<u8>>,
    ) -> Result<(String, UnboundedSender<Vec<u8>>)> {
        let exec = self
            .docker
            .create_exec(
                &self.container_name,
                CreateExecOptions {
                    cmd: Some(cmd),
                    attach_stdin: Some(true),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    tty: Some(true),
                    ..Default::default()
                },
            )
            .await
            .context("interactive exec create failed")?;
        let exec_id = exec.id.clone();

        let StartExecResults::Attached {
            mut output,
            mut input,
        } = self
            .docker
            .start_exec(&exec.id, None)
            .await
            .context("interactive exec start failed")?
        else {
            anyhow::bail!("docker returned a detached exec for an interactive session");
        };

        tokio::spawn(async move {
            while let Some(chunk) = output.next().await {
                match chunk {
                    Ok(log) => {
                        if on_output.send(log.into_bytes().to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let (input_tx, mut input_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
        tokio::spawn(async move {
            while let Some(bytes) = input_rx.recv().await {
                if input.write_all(&bytes).await.is_err() {
                    break;
                }
            }
            let _ = input.shutdown().await;
        });

        Ok((exec_id, input_tx))
    }

    /// Resizes the pty backing an interactive exec session (call on terminal resize).
    pub async fn resize_exec(&self, exec_id: &str, cols: u16, rows: u16) -> Result<()> {
        self.docker
            .resize_exec(
                exec_id,
                ResizeExecOptions {
                    height: rows,
                    width: cols,
                },
            )
            .await
            .context("resize_exec failed")?;
        Ok(())
    }

    /// Fire-and-wait helper for short control commands (e.g. supervisorctl tor toggles).
    pub async fn exec_oneshot(&self, cmd: String) -> Result<String> {
        let exec = self
            .docker
            .create_exec(
                &self.container_name,
                CreateExecOptions {
                    cmd: Some(vec!["sh".to_string(), "-c".to_string(), cmd]),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    ..Default::default()
                },
            )
            .await
            .context("exec create failed")?;

        let mut out = String::new();
        if let StartExecResults::Attached { mut output, .. } = self
            .docker
            .start_exec(&exec.id, None)
            .await
            .context("exec start failed")?
        {
            while let Some(chunk) = output.next().await {
                if let Ok(log) = chunk {
                    out.push_str(&log.to_string());
                }
            }
        }
        Ok(out)
    }
}
