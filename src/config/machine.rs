use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Machine-local identity: which roles this machine should apply.
/// Stored in ~/.local/share/qwert/machine.yml; overridden by QWERT_ROLES env var.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct MachineIdentity {
    /// Ordered role names — order defines override precedence (last wins).
    #[serde(default)]
    pub roles: Vec<String>,
}

impl MachineIdentity {
    /// Load from env override (QWERT_ROLES, comma-separated) or machine.yml.
    pub fn load() -> Result<Self> {
        if let Ok(env) = std::env::var("QWERT_ROLES") {
            return Ok(Self::from_env(&env));
        }
        Self::load_from(&machine_path())
    }

    pub fn save(&self) -> Result<()> {
        self.save_to(&machine_path())
    }

    pub fn set_roles(&mut self, roles: Vec<String>) {
        self.roles = roles;
    }

    /// Parse a comma-separated role list (QWERT_ROLES override).
    pub fn from_env(env: &str) -> Self {
        let roles: Vec<String> = env
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        Self { roles }
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        Ok(serde_yml::from_str(&content)?)
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_yml::to_string(self)?)?;
        Ok(())
    }
}

/// Path to the machine identity file: ~/.local/share/qwert/machine.yml
pub fn machine_path() -> PathBuf {
    crate::platform::data_dir().join("machine.yml")
}

#[cfg(test)]
#[path = "tests/machine.rs"]
mod tests;