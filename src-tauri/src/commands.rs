use base64::Engine;
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::mpsc;

use cyberbox_core::ai;
use cyberbox_core::docker::OutputEvent;
use cyberbox_core::registry::ToolSpec;
use cyberbox_core::tor;

use crate::state::{AppState, InstallStatus, PtySession};

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn b64_decode(s: &str) -> Vec<u8> {
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .unwrap_or_default()
}

#[derive(Serialize)]
pub struct ToolInfo {
    #[serde(flatten)]
    pub spec: ToolSpec,
    pub install_status: InstallStatus,
}

#[tauri::command]
pub async fn list_tools(state: State<'_, AppState>) -> Result<Vec<ToolInfo>, String> {
    let statuses = state.install_status.lock().await;
    Ok(state
        .registry
        .tools
        .iter()
        .map(|spec| ToolInfo {
            spec: spec.clone(),
            install_status: statuses
                .get(&spec.name)
                .cloned()
                .unwrap_or(InstallStatus::Unknown),
        })
        .collect())
}

#[tauri::command]
pub async fn container_status(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.docker.is_running().await)
}

#[tauri::command]
pub async fn tor_status(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(tor::is_running(&state.docker).await)
}

#[tauri::command]
pub async fn toggle_tor(state: State<'_, AppState>, enabled: bool) -> Result<bool, String> {
    tor::set_enabled(&state.docker, enabled)
        .await
        .map_err(|e| e.to_string())?;
    Ok(tor::is_running(&state.docker).await)
}

/// Batch-checks which registered tools' binaries exist in the container.
#[tauri::command]
pub async fn refresh_install_status(state: State<'_, AppState>) -> Result<(), String> {
    if state.registry.tools.is_empty() {
        return Ok(());
    }
    let probe = state
        .registry
        .tools
        .iter()
        .map(|t| {
            format!(
                "command -v {0} >/dev/null 2>&1 && echo '{0}:yes' || echo '{0}:no'",
                t.binary
            )
        })
        .collect::<Vec<_>>()
        .join("; ");

    let out = state
        .docker
        .exec_oneshot(probe)
        .await
        .map_err(|e| e.to_string())?;
    let mut statuses = state.install_status.lock().await;
    for line in out.lines() {
        if let Some((binary, result)) = line.rsplit_once(':') {
            if let Some(tool) = state.registry.tools.iter().find(|t| t.binary == binary) {
                let status = if result.trim() == "yes" {
                    InstallStatus::Installed
                } else {
                    InstallStatus::NotInstalled
                };
                statuses.insert(tool.name.clone(), status);
            }
        }
    }
    Ok(())
}

/// Installs a tool via its `install_cmd`, streaming progress as
/// `install:{name}:line` events and finishing with `install:{name}:done`.
#[tauri::command]
pub async fn install_tool(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    let Some(tool) = state
        .registry
        .tools
        .iter()
        .find(|t| t.name == name)
        .cloned()
    else {
        return Err(format!("unknown tool: {name}"));
    };
    let Some(install_cmd) = tool.install_cmd.clone() else {
        return Err(format!("{name} ships pre-installed — nothing to install"));
    };

    {
        let mut statuses = state.install_status.lock().await;
        statuses.insert(name.clone(), InstallStatus::Installing);
    }

    let (tx, mut rx) = mpsc::unbounded_channel();
    state.docker.exec_stream(install_cmd, tx);

    let mut success = false;
    while let Some(event) = rx.recv().await {
        match event {
            OutputEvent::Line(line) => {
                let _ = app.emit(&format!("install:{name}:line"), line);
            }
            OutputEvent::Done { exit_code } => {
                success = exit_code == Some(0);
            }
            OutputEvent::Error(e) => {
                let _ = app.emit(&format!("install:{name}:line"), format!("error: {e}"));
            }
        }
    }

    let final_status = if success {
        InstallStatus::Installed
    } else {
        InstallStatus::Failed("install failed".to_string())
    };
    {
        let mut statuses = state.install_status.lock().await;
        statuses.insert(name.clone(), final_status.clone());
    }
    let _ = app.emit(&format!("install:{name}:done"), success);
    Ok(())
}

/// Lists entries in a container directory (`ls -1p`); returns names with a
/// trailing `/` for directories, matching `ls -p` conventions.
#[tauri::command]
pub async fn list_dir(state: State<'_, AppState>, path: String) -> Result<Vec<String>, String> {
    let safe_path = path.replace('\'', "");
    let out = state
        .docker
        .exec_oneshot(format!("ls -1p '{safe_path}' 2>/dev/null"))
        .await
        .map_err(|e| e.to_string())?;
    Ok(out.lines().map(|l| l.to_string()).collect())
}

/// Reads a file from the container, returning its content base64-encoded
/// (binary-safe, and keeps large payloads out of shell-escaping concerns).
#[tauri::command]
pub async fn read_file(state: State<'_, AppState>, path: String) -> Result<String, String> {
    let safe_path = path.replace('\'', "");
    let out = state
        .docker
        .exec_oneshot(format!(
            "[ -f '{safe_path}' ] && base64 -w0 '{safe_path}' || echo __cyberbox_missing__"
        ))
        .await
        .map_err(|e| e.to_string())?;
    let out = out.trim();
    if out == "__cyberbox_missing__" {
        return Err(format!("{path} does not exist or is not a regular file"));
    }
    Ok(out.to_string())
}

/// Writes base64-encoded `content` to a file in the container, overwriting it.
#[tauri::command]
pub async fn write_file(
    state: State<'_, AppState>,
    path: String,
    content: String,
) -> Result<(), String> {
    let safe_path = path.replace('\'', "");
    // `content` is base64 (alphabet A-Za-z0-9+/=), safe to embed in single quotes.
    let cmd = format!("printf '%s' '{content}' | base64 -d > '{safe_path}'");
    let out = state
        .docker
        .exec_oneshot(cmd)
        .await
        .map_err(|e| e.to_string())?;
    if !out.trim().is_empty() {
        return Err(out.trim().to_string());
    }
    Ok(())
}

/// Opens a real interactive shell in the toolbox container for tab `id`.
/// Output bytes are forwarded as base64 via the `pty:{id}:data` event.
#[tauri::command]
pub async fn pty_open(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    // `id` is always a UUID from the frontend (crypto.randomUUID()), safe to
    // interpolate directly. We track the shell's cwd by appending an
    // invisible `$(pwd > file)` command substitution to PS1 (PS1 is
    // re-expanded on every prompt draw, same as PROMPT_COMMAND) — read by
    // `pty_cwd` below to drive the Files sidebar.
    //
    // This does NOT use PROMPT_COMMAND: Kali's own /root/.bashrc sets
    // NEWLINE_BEFORE_PROMPT=yes, which makes it run
    // `PROMPT_COMMAND="PROMPT_COMMAND=echo"` on every prompt — a
    // self-mutating idiom that unconditionally clobbers whatever
    // PROMPT_COMMAND we set beforehand. PS1 has no such conflict.
    //
    // Reading /proc/<pid>/cwd instead doesn't work either: docker's exec
    // inspect reports the host-visible pid, but the shell runs as root
    // inside the container, so an unprivileged host process can't read its
    // /proc/<pid>/cwd (EACCES), and resolving it from *inside* the
    // container hits a different, unrelated pid in that mount/pid
    // namespace.
    let cwd_file = format!("/tmp/.cyberbox-cwd-{id}");
    let rc_file = format!("/tmp/.cyberbox-rc-{id}");
    let rc_contents = format!(
        "[ -f /root/.bashrc ] && source /root/.bashrc\nPS1=\"$PS1\\$(pwd > {cwd_file} 2>/dev/null)\"\n"
    );
    let write_rc = format!("cat > '{rc_file}' << 'CYBERBOX_EOF'\n{rc_contents}CYBERBOX_EOF\n");
    state
        .docker
        .exec_oneshot(write_rc)
        .await
        .map_err(|e| e.to_string())?;

    tracing::info!(%id, %rc_file, %cwd_file, "pty_open: spawning interactive shell");
    let (exec_id, input) = state
        .docker
        .exec_interactive(
            vec![
                "/bin/bash".to_string(),
                "--rcfile".to_string(),
                rc_file,
                "-i".to_string(),
            ],
            out_tx,
        )
        .await
        .map_err(|e| e.to_string())?;
    tracing::info!(%id, %exec_id, "pty_open: shell spawned");

    state
        .pty_sessions
        .lock()
        .await
        .insert(id.clone(), PtySession { input, exec_id });

    let app_handle = app.clone();
    let tab_id = id.clone();
    tokio::spawn(async move {
        while let Some(bytes) = out_rx.recv().await {
            let _ = app_handle.emit(&format!("pty:{tab_id}:data"), b64(&bytes));
        }
        let _ = app_handle.emit(&format!("pty:{tab_id}:exit"), ());
    });

    Ok(())
}

#[tauri::command]
pub async fn pty_write(state: State<'_, AppState>, id: String, data: String) -> Result<(), String> {
    let sessions = state.pty_sessions.lock().await;
    let Some(session) = sessions.get(&id) else {
        return Err(format!("no pty session for tab {id}"));
    };
    session
        .input
        .send(b64_decode(&data))
        .map_err(|_| "pty session closed".to_string())
}

#[tauri::command]
pub async fn pty_resize(
    state: State<'_, AppState>,
    id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let exec_id = {
        let sessions = state.pty_sessions.lock().await;
        sessions.get(&id).map(|s| s.exec_id.clone())
    };
    let Some(exec_id) = exec_id else {
        return Err(format!("no pty session for tab {id}"));
    };
    state
        .docker
        .resize_exec(&exec_id, cols, rows)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pty_close(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.pty_sessions.lock().await.remove(&id);
    let _ = state
        .docker
        .exec_oneshot(format!(
            "rm -f /tmp/.cyberbox-cwd-{id} /tmp/.cyberbox-rc-{id}"
        ))
        .await;
    Ok(())
}

/// Returns the live current working directory of tab `id`'s shell, so the
/// Files sidebar can follow wherever the user has `cd`'d to. Backed by the
/// PROMPT_COMMAND file written in `pty_open`, not /proc introspection.
#[tauri::command]
pub async fn pty_cwd(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let out = state
        .docker
        .exec_oneshot(format!("cat /tmp/.cyberbox-cwd-{id} 2>/dev/null"))
        .await
        .map_err(|e| e.to_string())?;
    let cwd = out.trim();
    tracing::info!(%id, raw = %out, %cwd, "pty_cwd: read result");
    if cwd.is_empty() {
        return Err("cwd not available yet".to_string());
    }
    Ok(cwd.to_string())
}

/// Streams an AI response as `ai:{request_id}:token` / `:done` / `:error` events.
#[tauri::command]
pub async fn ask_ai(
    app: AppHandle,
    state: State<'_, AppState>,
    request_id: String,
    question: String,
    context: String,
) -> Result<(), String> {
    let (tx, mut rx) = mpsc::unbounded_channel();
    ai::ask(
        state.config.ollama_url.clone(),
        state.config.ollama_model.clone(),
        question,
        context,
        tx,
    );

    while let Some(event) = rx.recv().await {
        match event {
            ai::AiEvent::Token(t) => {
                let _ = app.emit(&format!("ai:{request_id}:token"), t);
            }
            ai::AiEvent::Done => {
                let _ = app.emit(&format!("ai:{request_id}:done"), ());
            }
            ai::AiEvent::Error(e) => {
                let _ = app.emit(&format!("ai:{request_id}:error"), e);
            }
        }
    }
    Ok(())
}
