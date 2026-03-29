use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Tracks what qwert has installed on this machine.
/// Used by `apply` to uninstall tools removed from qwert.yml.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct QwertState {
    #[serde(default)]
    pub installed: Vec<String>,
}

impl QwertState {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        Ok(serde_yml::from_str(&content)?)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_yml::to_string(self)?)?;
        Ok(())
    }

    pub fn mark_installed(&mut self, name: &str) {
        if !self.installed.iter().any(|t| t == name) {
            self.installed.push(name.to_string());
        }
    }

    pub fn mark_removed(&mut self, name: &str) {
        self.installed.retain(|t| t != name);
    }

    /// Tools in state but not in the manifest — should be uninstalled.
    pub fn orphans(&self, manifest_tools: &[String]) -> Vec<&str> {
        self.installed
            .iter()
            .filter(|t| !manifest_tools.iter().any(|m| m == *t))
            .map(|t| t.as_str())
            .collect()
    }
}

pub fn state_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".qwert")
        .join("state.yml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mark_installed_adds_tool() {
        // arrange
        let mut state = QwertState::default();
        // act
        state.mark_installed("tmux");
        // assert
        assert!(state.installed.contains(&"tmux".to_string()));
    }

    #[test]
    fn mark_installed_ignores_duplicate() {
        // arrange
        let mut state = QwertState::default();
        state.mark_installed("tmux");
        // act
        state.mark_installed("tmux");
        // assert
        assert_eq!(state.installed.len(), 1);
    }

    #[test]
    fn mark_removed_deletes_tool() {
        // arrange
        let mut state = QwertState::default();
        state.mark_installed("tmux");
        state.mark_installed("neovim");
        // act
        state.mark_removed("tmux");
        // assert
        assert!(!state.installed.contains(&"tmux".to_string()));
        assert!(state.installed.contains(&"neovim".to_string()));
    }

    #[test]
    fn mark_removed_is_noop_when_absent() {
        // arrange
        let mut state = QwertState::default();
        state.mark_installed("neovim");
        // act
        state.mark_removed("tmux");
        // assert
        assert_eq!(state.installed.len(), 1);
    }

    #[test]
    fn orphans_returns_tools_not_in_manifest() {
        // arrange
        let mut state = QwertState::default();
        state.mark_installed("tmux");
        state.mark_installed("neovim");
        state.mark_installed("delta");
        let manifest = vec!["neovim".to_string()];
        // act
        let orphans = state.orphans(&manifest);
        // assert
        assert_eq!(orphans.len(), 2);
        assert!(orphans.contains(&"tmux"));
        assert!(orphans.contains(&"delta"));
    }

    #[test]
    fn orphans_returns_empty_when_all_in_manifest() {
        // arrange
        let mut state = QwertState::default();
        state.mark_installed("tmux");
        let manifest = vec!["tmux".to_string()];
        // act
        let orphans = state.orphans(&manifest);
        // assert
        assert!(orphans.is_empty());
    }

    #[test]
    fn save_and_load_roundtrip() {
        // arrange
        let mut state = QwertState::default();
        state.mark_installed("tmux");
        state.mark_installed("delta");
        let path = std::env::temp_dir().join("qwert_test_state.yml");
        // act
        state.save(&path).unwrap();
        let loaded = QwertState::load(&path).unwrap();
        std::fs::remove_file(&path).ok();
        // assert
        assert!(loaded.installed.contains(&"tmux".to_string()));
        assert!(loaded.installed.contains(&"delta".to_string()));
        assert_eq!(loaded.installed.len(), 2);
    }

    #[test]
    fn load_returns_default_when_file_missing() {
        // arrange
        let path = std::env::temp_dir().join("qwert_state_nonexistent.yml");
        // act
        let state = QwertState::load(&path).unwrap();
        // assert
        assert!(state.installed.is_empty());
    }
}
