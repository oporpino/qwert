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
    pub hooks: Hooks,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Hooks {
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
        serde_yml::from_str(&content)
            .with_context(|| format!("failed to parse {}", path.display()))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_yml::to_string(self)?;
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

    pub fn add_hook(&mut self, hook: &str, path: &str) {
        let scripts = match hook {
            "init" => &mut self.hooks.init,
            "end" => &mut self.hooks.end,
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
        return home.join(".config");
    }
    PathBuf::from("~/.config")
}

/// Path to the qwert.yml manifest
pub fn manifest_path() -> PathBuf {
    config_dir().join("qwert.yml")
}

pub(crate) fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return format!("{}/{}", home.display(), &path[2..]);
        }
    }
    path.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_tool_appends_new_tool() {
        // arrange
        let mut config = QwertConfig::default();
        // act
        config.add_tool("neovim");
        // assert
        assert_eq!(config.tools, vec!["neovim"]);
    }

    #[test]
    fn add_tool_ignores_duplicate() {
        // arrange
        let mut config = QwertConfig::default();
        config.add_tool("neovim");
        // act
        config.add_tool("neovim");
        // assert
        assert_eq!(config.tools.len(), 1);
    }

    #[test]
    fn remove_tool_deletes_existing_tool() {
        // arrange
        let mut config = QwertConfig::default();
        config.add_tool("neovim");
        config.add_tool("tmux");
        // act
        config.remove_tool("neovim");
        // assert
        assert_eq!(config.tools, vec!["tmux"]);
    }

    #[test]
    fn remove_tool_is_noop_when_absent() {
        // arrange
        let mut config = QwertConfig::default();
        config.add_tool("tmux");
        // act
        config.remove_tool("neovim");
        // assert
        assert_eq!(config.tools, vec!["tmux"]);
    }

    #[test]
    fn has_tool_returns_true_when_present() {
        // arrange
        let mut config = QwertConfig::default();
        config.add_tool("tmux");
        // act
        let result = config.has_tool("tmux");
        // assert
        assert!(result);
    }

    #[test]
    fn has_tool_returns_false_when_absent() {
        // arrange
        let config = QwertConfig::default();
        // act
        let result = config.has_tool("tmux");
        // assert
        assert!(!result);
    }

    #[test]
    fn add_hook_appends_to_init_hook() {
        // arrange
        let mut config = QwertConfig::default();
        // act
        config.add_hook("init", "~/dotfiles/env.sh");
        // assert
        assert_eq!(config.hooks.init, vec!["~/dotfiles/env.sh"]);
    }

    #[test]
    fn add_hook_appends_to_end_hook() {
        // arrange
        let mut config = QwertConfig::default();
        // act
        config.add_hook("end", "~/dotfiles/aliases.sh");
        // assert
        assert_eq!(config.hooks.end, vec!["~/dotfiles/aliases.sh"]);
    }

    #[test]
    fn add_hook_ignores_duplicate_path() {
        // arrange
        let mut config = QwertConfig::default();
        config.add_hook("init", "~/env.sh");
        // act
        config.add_hook("init", "~/env.sh");
        // assert
        assert_eq!(config.hooks.init.len(), 1);
    }

    #[test]
    fn add_hook_ignores_unknown_hook() {
        // arrange
        let mut config = QwertConfig::default();
        // act
        config.add_hook("unknown", "~/script.sh");
        // assert — no panic, no side effects
        assert!(config.hooks.init.is_empty());
        assert!(config.hooks.end.is_empty());
    }

    #[test]
    fn save_and_load_roundtrip() {
        // arrange
        let mut config = QwertConfig::default();
        config.add_tool("tmux");
        config.add_tool("neovim");
        config.add_hook("init", "~/env.sh");
        let path = std::env::temp_dir().join("qwert_test_roundtrip.yml");
        // act
        config.save(&path).unwrap();
        let loaded = QwertConfig::load(&path).unwrap();
        std::fs::remove_file(&path).ok();
        // assert
        assert_eq!(loaded.tools, vec!["tmux", "neovim"]);
        assert_eq!(loaded.hooks.init, vec!["~/env.sh"]);
    }

    #[test]
    fn load_returns_default_when_file_missing() {
        // arrange
        let path = std::env::temp_dir().join("qwert_nonexistent_xyz.yml");
        // act
        let config = QwertConfig::load(&path).unwrap();
        // assert
        assert!(config.tools.is_empty());
    }

    #[test]
    fn config_dir_uses_env_var_when_set() {
        // arrange
        std::env::set_var("QWERT_CONFIG_DIR", "/tmp/my-dotfiles");
        // act
        let dir = config_dir();
        std::env::remove_var("QWERT_CONFIG_DIR");
        // assert
        assert_eq!(dir, std::path::PathBuf::from("/tmp/my-dotfiles"));
    }

    #[test]
    fn expand_tilde_replaces_prefix() {
        // arrange
        let home = dirs::home_dir().unwrap();
        // act
        let result = expand_tilde("~/dotfiles/env.sh");
        // assert
        assert!(result.starts_with(home.to_str().unwrap()));
        assert!(result.ends_with("dotfiles/env.sh"));
    }

    #[test]
    fn expand_tilde_leaves_absolute_path_unchanged() {
        // arrange
        let path = "/etc/profile";
        // act
        let result = expand_tilde(path);
        // assert
        assert_eq!(result, "/etc/profile");
    }
}
