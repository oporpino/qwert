use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct QwertConfig {
    #[serde(default)]
    pub tools: Vec<String>,

    #[serde(default)]
    pub stacks: Vec<String>,

    #[serde(default)]
    pub scripts: Scripts,

    #[serde(default)]
    pub configs: std::collections::HashMap<String, String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Scripts {
    #[serde(default)]
    pub init: Vec<String>,

    #[serde(default)]
    pub end: Vec<String>,
}

impl QwertConfig {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        serde_yaml::from_str(&content)
            .with_context(|| format!("failed to parse {}", path.display()))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn add_tool(&mut self, name: &str) {
        if !self.tools.iter().any(|t| t == name) {
            self.tools.push(name.to_string());
        }
    }

    pub fn remove_tool(&mut self, name: &str) {
        self.tools.retain(|t| t != name);
    }

    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.iter().any(|t| t == name)
    }

    pub fn add_script(&mut self, hook: &str, path: &str) {
        let scripts = match hook {
            "init" => &mut self.scripts.init,
            "end" => &mut self.scripts.end,
            _ => return,
        };
        if !scripts.iter().any(|s| s == path) {
            scripts.push(path.to_string());
        }
    }
}

/// Resolve the config directory from env or default
pub fn config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("QWERT_CONFIG_DIR") {
        return PathBuf::from(dir);
    }
    // Also check ~/.qwert/config for the persisted setting
    if let Some(home) = dirs::home_dir() {
        let cfg_file = home.join(".qwert").join("config");
        if cfg_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&cfg_file) {
                for line in content.lines() {
                    if let Some(val) = line.strip_prefix("QWERT_CONFIG_DIR=") {
                        let expanded = expand_tilde(val.trim());
                        return PathBuf::from(expanded);
                    }
                }
            }
        }
        return home.join(".config").join("qwert");
    }
    PathBuf::from("~/.config/qwert")
}

/// Path to the qwert.yml manifest
pub fn manifest_path() -> PathBuf {
    config_dir().join("qwert.yml")
}

fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return format!("{}/{}", home.display(), &path[2..]);
        }
    }
    path.to_string()
}
