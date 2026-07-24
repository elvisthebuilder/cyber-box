use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc::UnboundedReceiver;

use cyberbox_core::ai::{self, AiEvent};
use cyberbox_core::config::Config;
use cyberbox_core::docker::DockerClient;
use cyberbox_core::registry::{Registry, ToolSpec};
use cyberbox_core::session::{SessionStatus, ToolSession};
use cyberbox_core::tor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallStatus {
    Unknown,
    Installed,
    NotInstalled,
    Installing,
    Failed(String),
}

#[derive(Debug, Clone)]
enum SessionKind {
    ToolRun,
    Install { tool_name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Browser,
    Output,
    Ai,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    TargetPrompt,
    AiInput,
}

pub struct FlatEntry {
    pub category: String,
    pub tool_index: Option<usize>, // None = category header row
}

pub struct App {
    pub config: Config,
    pub docker: Arc<DockerClient>,
    pub registry: Registry,
    pub flat_list: Vec<FlatEntry>,
    pub selected: usize,

    pub focus: Focus,
    pub input_mode: InputMode,
    pub should_quit: bool,

    pub container_running: bool,
    pub tor_enabled: bool,
    pub ai_enabled: bool,

    pub session: Option<ToolSession>,
    session_kind: Option<SessionKind>,
    pub output_scroll: u16,
    pub install_status: HashMap<String, InstallStatus>,

    pub ai_lines: Vec<String>,
    pub ai_input: String,
    pub ai_rx: Option<UnboundedReceiver<AiEvent>>,
    pub ai_busy: bool,

    pub target_input: String,
    pub pending_tool: Option<usize>,

    pub status_message: Option<String>,
    pub last_poll: Instant,
}

impl App {
    pub fn new(config: Config, docker: Arc<DockerClient>, registry: Registry) -> Self {
        let flat_list = build_flat_list(&registry);
        let install_status = registry
            .tools
            .iter()
            .map(|t| (t.name.clone(), InstallStatus::Unknown))
            .collect();
        Self {
            config,
            docker,
            registry,
            flat_list,
            selected: 0,
            focus: Focus::Browser,
            input_mode: InputMode::Normal,
            should_quit: false,
            container_running: false,
            tor_enabled: false,
            ai_enabled: true,
            session: None,
            session_kind: None,
            output_scroll: 0,
            install_status,
            ai_lines: Vec::new(),
            ai_input: String::new(),
            ai_rx: None,
            ai_busy: false,
            target_input: String::new(),
            pending_tool: None,
            status_message: None,
            last_poll: Instant::now() - Duration::from_secs(10),
        }
    }

    /// Public helper for future UI (e.g. a detail/preview panel); not yet
    /// called anywhere since the browser list already shows descriptions inline.
    #[allow(dead_code)]
    pub fn selected_tool(&self) -> Option<&ToolSpec> {
        let entry = self.flat_list.get(self.selected)?;
        let idx = entry.tool_index?;
        self.registry.tools.iter().filter(|t| t.category == entry.category).nth(idx)
    }

    pub fn tick(&mut self) {
        if let Some(session) = self.session.as_mut() {
            session.poll();
            let status = session.status;
            if let (Some(SessionKind::Install { tool_name }), SessionStatus::Finished { .. } | SessionStatus::Failed) =
                (&self.session_kind, status)
            {
                let new_status = match status {
                    SessionStatus::Finished { exit_code: Some(0) } => InstallStatus::Installed,
                    SessionStatus::Finished { exit_code } => InstallStatus::Failed(format!("exit code {exit_code:?}")),
                    SessionStatus::Failed => InstallStatus::Failed("install exec failed".to_string()),
                    SessionStatus::Running => unreachable!(),
                };
                self.install_status.insert(tool_name.clone(), new_status);
                self.session_kind = None;
            }
        }
        if let Some(rx) = self.ai_rx.as_mut() {
            while let Ok(event) = rx.try_recv() {
                match event {
                    AiEvent::Token(t) => {
                        if let Some(last) = self.ai_lines.last_mut() {
                            last.push_str(&t);
                        }
                    }
                    AiEvent::Done => {
                        self.ai_busy = false;
                    }
                    AiEvent::Error(e) => {
                        self.ai_lines.push(format!("[error] {e}"));
                        self.ai_busy = false;
                    }
                }
            }
        }

    }

    pub fn status_due(&self) -> bool {
        self.last_poll.elapsed() >= Duration::from_secs(2)
    }

    pub async fn refresh_status(&mut self) {
        self.last_poll = Instant::now();
        self.container_running = self.docker.is_running().await;
        self.tor_enabled = tor::is_running(&self.docker).await;
    }

    pub async fn handle_key(&mut self, key: KeyEvent) {
        // Global keybindings.
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('c') => {
                    self.should_quit = true;
                    return;
                }
                KeyCode::Char('t') => {
                    self.toggle_tor().await;
                    return;
                }
                KeyCode::Char('a') => {
                    self.ai_enabled = !self.ai_enabled;
                    return;
                }
                _ => {}
            }
        }

        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key).await,
            InputMode::TargetPrompt => self.handle_target_prompt_key(key).await,
            InputMode::AiInput => self.handle_ai_input_key(key).await,
        }
    }

    async fn handle_normal_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Focus::Browser => Focus::Output,
                    Focus::Output => Focus::Ai,
                    Focus::Ai => Focus::Browser,
                };
                if self.focus == Focus::Ai {
                    self.input_mode = InputMode::AiInput;
                }
            }
            KeyCode::Esc => {
                self.focus = Focus::Browser;
                self.input_mode = InputMode::Normal;
            }
            _ => match self.focus {
                Focus::Browser => self.handle_browser_key(key),
                Focus::Output => self.handle_output_key(key),
                Focus::Ai => {}
            },
        }
    }

    fn handle_browser_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down if self.selected + 1 < self.flat_list.len() => {
                self.selected += 1;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Enter => {
                if let Some(entry) = self.flat_list.get(self.selected) {
                    if let Some(idx) = entry.tool_index {
                        let not_installed = self
                            .registry
                            .tools
                            .iter()
                            .filter(|t| t.category == entry.category)
                            .nth(idx)
                            .map(|t| {
                                matches!(
                                    self.install_status.get(&t.name),
                                    Some(InstallStatus::NotInstalled) | Some(InstallStatus::Installing)
                                )
                            })
                            .unwrap_or(false);
                        if not_installed {
                            self.status_message =
                                Some("not installed yet — press 'i' to install it first".to_string());
                            return;
                        }
                        self.pending_tool = Some(self.selected);
                        self.target_input.clear();
                        self.input_mode = InputMode::TargetPrompt;
                    }
                }
            }
            KeyCode::Char('i') => self.install_selected(),
            _ => {}
        }
    }

    fn handle_output_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.output_scroll = self.output_scroll.saturating_add(1),
            KeyCode::Char('k') | KeyCode::Up => self.output_scroll = self.output_scroll.saturating_sub(1),
            _ => {}
        }
    }

    async fn handle_target_prompt_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.pending_tool = None;
            }
            KeyCode::Enter => {
                self.launch_pending_tool();
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Backspace => {
                self.target_input.pop();
            }
            KeyCode::Char(c) => self.target_input.push(c),
            _ => {}
        }
    }

    fn launch_pending_tool(&mut self) {
        let Some(sel) = self.pending_tool.take() else { return };
        let Some(entry) = self.flat_list.get(sel) else { return };
        let Some(idx) = entry.tool_index else { return };
        let Some(tool) = self
            .registry
            .tools
            .iter()
            .filter(|t| t.category == entry.category)
            .nth(idx)
            .cloned()
        else {
            return;
        };

        let target = if self.target_input.trim().is_empty() {
            "127.0.0.1".to_string()
        } else {
            self.target_input.clone()
        };
        let cmd = Registry::build_command(&tool, &target, None);
        let (cmd, warning) = tor::maybe_wrap(cmd, self.tor_enabled, tool.tor_wrappable);
        if let Some(w) = warning {
            self.status_message = Some(w);
        }

        self.output_scroll = 0;
        self.session_kind = Some(SessionKind::ToolRun);
        self.session = Some(ToolSession::launch(&self.docker, tool.name.clone(), cmd));
    }

    /// Installs the currently selected tool in the background via its
    /// `install_cmd`, streaming progress into the Output Pane like a normal
    /// tool run. No-op if the tool has no `install_cmd` or is already
    /// installed/installing.
    fn install_selected(&mut self) {
        let Some(entry) = self.flat_list.get(self.selected) else { return };
        let Some(idx) = entry.tool_index else { return };
        let Some(tool) = self
            .registry
            .tools
            .iter()
            .filter(|t| t.category == entry.category)
            .nth(idx)
            .cloned()
        else {
            return;
        };

        let Some(install_cmd) = tool.install_cmd.clone() else {
            self.status_message = Some(format!("{} ships pre-installed — nothing to install", tool.name));
            return;
        };

        match self.install_status.get(&tool.name) {
            Some(InstallStatus::Installed) => {
                self.status_message = Some(format!("{} is already installed", tool.name));
                return;
            }
            Some(InstallStatus::Installing) => {
                self.status_message = Some(format!("{} is already installing", tool.name));
                return;
            }
            _ => {}
        }

        self.install_status.insert(tool.name.clone(), InstallStatus::Installing);
        self.output_scroll = 0;
        self.session_kind = Some(SessionKind::Install { tool_name: tool.name.clone() });
        self.session = Some(ToolSession::launch(
            &self.docker,
            format!("install: {}", tool.name),
            install_cmd,
        ));
    }

    /// Checks which registered tools' binaries already exist in the toolbox
    /// container, in a single batched exec. Called once at startup and can
    /// be re-run any time the container comes back up.
    pub async fn check_installed_status(&mut self) {
        if self.registry.tools.is_empty() {
            return;
        }
        let probe = self
            .registry
            .tools
            .iter()
            .map(|t| format!("command -v {0} >/dev/null 2>&1 && echo '{0}:yes' || echo '{0}:no'", t.binary))
            .collect::<Vec<_>>()
            .join("; ");

        if let Ok(out) = self.docker.exec_oneshot(probe).await {
            for line in out.lines() {
                if let Some((binary, state)) = line.rsplit_once(':') {
                    if let Some(tool) = self.registry.tools.iter().find(|t| t.binary == binary) {
                        let status = if state.trim() == "yes" { InstallStatus::Installed } else { InstallStatus::NotInstalled };
                        self.install_status.insert(tool.name.clone(), status);
                    }
                }
            }
        }
    }

    async fn handle_ai_input_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.focus = Focus::Browser;
            }
            KeyCode::Tab => {
                self.focus = Focus::Output;
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Enter => {
                if !self.ai_enabled {
                    self.ai_lines.push("[AI is disabled — press Ctrl+A to enable]".to_string());
                    self.ai_input.clear();
                    return;
                }
                if self.ai_input.trim().is_empty() {
                    return;
                }
                let question = self.ai_input.clone();
                self.ai_input.clear();
                self.ai_lines.push(format!("> {question}"));
                self.ai_lines.push(String::new()); // placeholder for streamed answer

                let context = self
                    .session
                    .as_ref()
                    .map(|s| s.lines.iter().rev().take(200).rev().cloned().collect::<Vec<_>>().join("\n"))
                    .unwrap_or_default();

                let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
                self.ai_rx = Some(rx);
                self.ai_busy = true;
                ai::ask(
                    self.config.ollama_url.clone(),
                    self.config.ollama_model.clone(),
                    question,
                    context,
                    tx,
                );
            }
            KeyCode::Backspace => {
                self.ai_input.pop();
            }
            KeyCode::Char(c) => self.ai_input.push(c),
            _ => {}
        }
    }

    async fn toggle_tor(&mut self) {
        let new_state = !self.tor_enabled;
        if tor::set_enabled(&self.docker, new_state).await.is_ok() {
            self.tor_enabled = new_state;
        } else {
            self.status_message = Some("failed to toggle Tor — is the toolbox container running?".to_string());
        }
    }
}

fn build_flat_list(registry: &Registry) -> Vec<FlatEntry> {
    let mut list = Vec::new();
    for (category, tools) in registry.by_category() {
        list.push(FlatEntry { category: category.clone(), tool_index: None });
        for i in 0..tools.len() {
            list.push(FlatEntry { category: category.clone(), tool_index: Some(i) });
        }
    }
    list
}
