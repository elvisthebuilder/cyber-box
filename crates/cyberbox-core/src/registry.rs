use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub category: String,
    pub description: String,
    pub binary: String,
    pub invocation: String,
    #[serde(default)]
    pub default_flags: String,
    #[serde(default = "default_output")]
    pub output: String,
    #[serde(default)]
    pub tor_wrappable: bool,
    /// Reserved for future sidecar-container tools (not used in v1).
    #[serde(default)]
    pub image: Option<String>,
    /// Shell command run inside the toolbox container to install this tool.
    /// Tools without this are assumed to ship baked into the toolbox image.
    #[serde(default)]
    pub install_cmd: Option<String>,
}

fn default_output() -> String {
    "stream".to_string()
}

#[derive(Debug, Deserialize)]
struct RawRegistry {
    #[serde(rename = "tool")]
    tools: Vec<ToolSpec>,
}

pub struct Registry {
    pub tools: Vec<ToolSpec>,
}

impl Registry {
    /// Loads `registry/tools.toml`-style relative path, searching upward from
    /// the current directory. Different frontends (the ratatui binary, the
    /// Tauri dev/build process) run with different working directories, so a
    /// bare relative path isn't reliable — this walks up a few parent
    /// directories to find the workspace root instead of hardcoding one.
    pub fn load(relative_path: impl AsRef<Path>) -> Result<Self> {
        let relative_path = relative_path.as_ref();
        let mut dir = std::env::current_dir().context("failed to read current directory")?;
        let path = loop {
            let candidate = dir.join(relative_path);
            if candidate.exists() {
                break candidate;
            }
            if !dir.pop() {
                anyhow::bail!(
                    "could not find {} by searching upward from the current directory",
                    relative_path.display()
                );
            }
        };

        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read registry file at {}", path.display()))?;
        let raw: RawRegistry = toml::from_str(&contents)
            .with_context(|| format!("failed to parse registry TOML at {}", path.display()))?;
        Ok(Self { tools: raw.tools })
    }

    /// Groups tools by category, preserving a stable sorted order for display.
    pub fn by_category(&self) -> BTreeMap<String, Vec<&ToolSpec>> {
        let mut map: BTreeMap<String, Vec<&ToolSpec>> = BTreeMap::new();
        for tool in &self.tools {
            map.entry(tool.category.clone()).or_default().push(tool);
        }
        map
    }

    /// Builds the final shell command for a tool given a target and optional
    /// flag override, substituting `{target}`/`{flags}` placeholders.
    pub fn build_command(tool: &ToolSpec, target: &str, flags_override: Option<&str>) -> String {
        let flags = flags_override.unwrap_or(&tool.default_flags);
        tool.invocation
            .replace("{target}", target)
            .replace("{flags}", flags)
    }
}
