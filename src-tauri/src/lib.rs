mod commands;
mod state;

use std::sync::Arc;

use cyberbox_core::config::Config;
use cyberbox_core::docker::DockerClient;
use cyberbox_core::registry::Registry;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt().init();

    let config = Config::default();
    let docker = match DockerClient::connect(&config) {
        Ok(d) => Arc::new(d),
        Err(e) => {
            eprintln!("fatal: could not connect to Docker daemon: {e}");
            std::process::exit(1);
        }
    };
    let registry = Registry::load(&config.registry_path).unwrap_or_else(|e| {
        eprintln!("warning: failed to load tool registry ({e}); starting with an empty toolset");
        Registry { tools: Vec::new() }
    });

    let state = AppState::new(config, docker.clone(), registry);

    tauri::Builder::default()
        .manage(state)
        .setup(move |_app| {
            let docker = docker.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = docker.ensure_container().await {
                    tracing::warn!("could not ensure toolbox container is running: {e}");
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_tools,
            commands::container_status,
            commands::tor_status,
            commands::toggle_tor,
            commands::refresh_install_status,
            commands::install_tool,
            commands::list_dir,
            commands::read_file,
            commands::write_file,
            commands::pty_open,
            commands::pty_write,
            commands::pty_resize,
            commands::pty_close,
            commands::pty_cwd,
            commands::list_ollama_models,
            commands::ask_ai,
        ])
        .run(tauri::generate_context!())
        .expect("error while running cyber-box");
}
