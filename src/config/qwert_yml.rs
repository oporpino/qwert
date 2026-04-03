use anyhow::{Context, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Backward-compatible tool entry: simple version string or full config object.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ToolEntry {
    Simple(String),
    Full(ToolConfig),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolConfig {
    #[serde(default = "default_version")]
    pub version: String,
    pub setup: Option<InlineSetup>,
}

fn default_version() -> String {
    "latest".into()
}

/// Inline setup defined in qwert.yml — mirrors RecipeSetup without importing recipe module.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InlineSetup {
    pub from: Option<String>,
    pub to: String,
    #[serde(default)]
    pub symlink: bool,
    pub macos: Option<StringOrList>,
    pub debian: Option<StringOrList>,
    pub undo: Option<InlineUndo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InlineUndo {
    pub macos: Option<StringOrList>,
    pub debian: Option<StringOrList>,
}

/// A single command string or an ordered list of commands (mirrors Commands in schema.rs).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum StringOrList {
    One(String),
    Many(Vec<String>),
}

impl StringOrList {
    #[allow(dead_code)]
    pub fn as_steps(&self) -> Vec<&str> {
        match self {
            StringOrList::One(s) => vec![s.as_str()],
            StringOrList::Many(v) => v.iter().map(|s| s.as_str()).collect(),
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct QwertConfig {
    /// name → version spec or full config (backward-compatible)
    #[serde(default)]
    pub tools: IndexMap<String, ToolEntry>,

    #[serde(default)]
    pub hooks: Hooks,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Hooks {
    /// Runs at the very top of .zshrc — only for things that must come first (e.g. p10k instant prompt)
    #[serde(default)]
    pub before: Vec<String>,

    /// Runs at the bottom of .zshrc — where most shell initialization happens
    #[serde(default)]
    pub init: Vec<String>,
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

    /// Add or update a tool. `version` defaults to "latest" if None.
    /// Preserves existing inline setup when updating an existing entry.
    pub fn add_tool(&mut self, name: &str, version: Option<&str>) {
        let ver = version.unwrap_or("latest").to_string();
        self.tools
            .entry(name.to_string())
            .and_modify(|e| match e {
                ToolEntry::Simple(v) => *v = ver.clone(),
                ToolEntry::Full(c) => c.version = ver.clone(),
            })
            .or_insert_with(|| ToolEntry::Simple(ver));
    }

    pub fn remove_tool(&mut self, name: &str) {
        self.tools.shift_remove(name);
    }

    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Ordered list of declared tool names.
    pub fn tool_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// Version spec for a tool ("latest" or semver). Returns "latest" if not declared.
    pub fn version_of(&self, name: &str) -> &str {
        match self.tools.get(name) {
            Some(ToolEntry::Simple(v)) => v.as_str(),
            Some(ToolEntry::Full(c)) => c.version.as_str(),
            None => "latest",
        }
    }

    /// Inline setup defined in qwert.yml for this tool, if any.
    pub fn setup_of(&self, name: &str) -> Option<&InlineSetup> {
        match self.tools.get(name) {
            Some(ToolEntry::Full(c)) => c.setup.as_ref(),
            _ => None,
        }
    }

    pub fn add_hook(&mut self, hook: &str, path: &str) {
        let scripts = match hook {
            "before" => &mut self.hooks.before,
            "init" => &mut self.hooks.init,
            _ => return,
        };
        if !scripts.iter().any(|s| s == path) {
            scripts.push(path.to_string());
        }
    }
}

/// User directory: ~/.qwert/ (dotfiles + manifest)
pub fn config_dir() -> PathBuf {
    dirs::home_dir()
        .expect("no home dir")
        .join(".qwert")
}

/// Path to the manifest: ~/.qwert/config.yml
pub fn manifest_path() -> PathBuf {
    config_dir().join("config.yml")
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
    fn add_tool_appends_new_tool_with_latest() {
        // arrange
        let mut config = QwertConfig::default();
        // act
        config.add_tool("neovim", None);
        // assert
        assert_eq!(config.version_of("neovim"), "latest");
    }

    #[test]
    fn add_tool_ignores_duplicate() {
        // arrange
        let mut config = QwertConfig::default();
        config.add_tool("neovim", None);
        // act
        config.add_tool("neovim", None);
        // assert
        assert_eq!(config.tools.len(), 1);
    }

    #[test]
    fn remove_tool_deletes_existing_tool() {
        // arrange
        let mut config = QwertConfig::default();
        config.add_tool("neovim", None);
        config.add_tool("tmux", None);
        // act
        config.remove_tool("neovim");
        // assert
        assert_eq!(config.tool_names(), vec!["tmux"]);
    }

    #[test]
    fn remove_tool_is_noop_when_absent() {
        // arrange
        let mut config = QwertConfig::default();
        config.add_tool("tmux", None);
        // act
        config.remove_tool("neovim");
        // assert
        assert_eq!(config.tool_names(), vec!["tmux"]);
    }

    #[test]
    fn has_tool_returns_true_when_present() {
        // arrange
        let mut config = QwertConfig::default();
        config.add_tool("tmux", None);
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
    fn add_hook_appends_to_init_hook_at_bottom() {
        // arrange
        let mut config = QwertConfig::default();
        // act
        config.add_hook("init", "~/dotfiles/aliases.sh");
        // assert
        assert_eq!(config.hooks.init, vec!["~/dotfiles/aliases.sh"]);
    }

    #[test]
    fn add_hook_ignores_duplicate_path() {
        // arrange
        let mut config = QwertConfig::default();
        config.add_hook("before", "~/env.sh");
        // act
        config.add_hook("before", "~/env.sh");
        // assert
        assert_eq!(config.hooks.before.len(), 1);
    }

    #[test]
    fn add_hook_ignores_unknown_hook() {
        // arrange
        let mut config = QwertConfig::default();
        // act
        config.add_hook("unknown", "~/script.sh");
        // assert — no panic, no side effects
        assert!(config.hooks.before.is_empty());
        assert!(config.hooks.init.is_empty());
    }

    #[test]
    fn save_and_load_roundtrip() {
        // arrange
        let mut config = QwertConfig::default();
        config.add_tool("tmux", None);
        config.add_tool("neovim", None);
        config.add_hook("init", "~/env.sh");
        let path = std::env::temp_dir().join("qwert_test_roundtrip.yml");
        // act
        config.save(&path).unwrap();
        let loaded = QwertConfig::load(&path).unwrap();
        std::fs::remove_file(&path).ok();
        // assert
        assert_eq!(loaded.tool_names(), vec!["tmux", "neovim"]);
        assert_eq!(loaded.version_of("tmux"), "latest");
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
    fn config_dir_returns_qwert_home() {
        // arrange
        let home = dirs::home_dir().unwrap();
        // act
        let dir = config_dir();
        // assert
        assert_eq!(dir, home.join(".qwert"));
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

    #[test]
    fn tool_entry_simple_parses_from_string() {
        // arrange
        let yaml = "tools:\n  tmux: latest\n";
        // act
        let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
        // assert
        assert_eq!(config.version_of("tmux"), "latest");
        assert!(config.setup_of("tmux").is_none());
    }

    #[test]
    fn tool_entry_full_parses_from_object() {
        // arrange
        let yaml = "tools:\n  neovim:\n    version: \"0.9\"\n";
        // act
        let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
        // assert
        assert_eq!(config.version_of("neovim"), "0.9");
        assert!(config.setup_of("neovim").is_none());
    }

    #[test]
    fn tool_entry_full_with_setup_symlink() {
        // arrange
        let yaml = "tools:\n  neovim:\n    version: latest\n    setup:\n      to: ~/.config/nvim\n      symlink: true\n";
        // act
        let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
        // assert
        let setup = config.setup_of("neovim").unwrap();
        assert_eq!(setup.to, "~/.config/nvim");
        assert!(setup.symlink);
    }

    #[test]
    fn tool_entry_full_with_commands() {
        // arrange
        let yaml = "tools:\n  delta:\n    version: latest\n    setup:\n      to: ~/.gitconfig\n      macos: \"git config --global core.pager delta\"\n";
        // act
        let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
        // assert
        let setup = config.setup_of("delta").unwrap();
        let steps = setup.macos.as_ref().unwrap().as_steps();
        assert_eq!(steps, vec!["git config --global core.pager delta"]);
    }

    #[test]
    fn setup_of_returns_none_for_simple_entry() {
        // arrange
        let mut config = QwertConfig::default();
        config.add_tool("tmux", None);
        // act + assert
        assert!(config.setup_of("tmux").is_none());
    }

    #[test]
    fn setup_of_returns_setup_for_full_entry() {
        // arrange
        let yaml = "tools:\n  tmux:\n    version: latest\n    setup:\n      to: ~/.tmux.conf\n      symlink: true\n";
        let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
        // act + assert
        assert!(config.setup_of("tmux").is_some());
    }

    #[test]
    fn version_of_returns_latest_for_simple_entry() {
        // arrange
        let mut config = QwertConfig::default();
        config.add_tool("tmux", None);
        // act + assert
        assert_eq!(config.version_of("tmux"), "latest");
    }

    #[test]
    fn version_of_returns_version_for_full_entry() {
        // arrange
        let yaml = "tools:\n  tmux:\n    version: \"3.4\"\n";
        let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
        // act + assert
        assert_eq!(config.version_of("tmux"), "3.4");
    }

    #[test]
    fn add_tool_preserves_existing_inline_setup() {
        // arrange
        let yaml = "tools:\n  neovim:\n    version: latest\n    setup:\n      to: ~/.config/nvim\n      symlink: true\n";
        let mut config: QwertConfig = serde_yml::from_str(yaml).unwrap();
        // act — update version without touching setup
        config.add_tool("neovim", Some("0.10"));
        // assert — setup must still be present
        assert_eq!(config.version_of("neovim"), "0.10");
        assert!(config.setup_of("neovim").is_some());
    }

    #[test]
    fn save_and_load_roundtrip_with_inline_setup() {
        // arrange
        let yaml = "tools:\n  tmux: latest\n  neovim:\n    version: latest\n    setup:\n      to: ~/.config/nvim\n      symlink: true\n";
        let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
        let path = std::env::temp_dir().join("qwert_test_inline_roundtrip.yml");
        // act
        config.save(&path).unwrap();
        let loaded = QwertConfig::load(&path).unwrap();
        std::fs::remove_file(&path).ok();
        // assert
        assert_eq!(loaded.version_of("tmux"), "latest");
        assert!(loaded.setup_of("tmux").is_none());
        let setup = loaded.setup_of("neovim").unwrap();
        assert_eq!(setup.to, "~/.config/nvim");
        assert!(setup.symlink);
    }
}
