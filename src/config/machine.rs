use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Machine-local identity: which profile this machine should apply.
/// A machine runs exactly one profile. Stored in ~/.local/share/qwert/machine.yml;
/// overridden by QWERT_PROFILE env var.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct MachineIdentity {
    /// The active profile for this machine. None = not configured yet.
    #[serde(default)]
    pub profile: Option<String>,
}

impl MachineIdentity {
    /// Load from env override (QWERT_PROFILE) or machine.yml.
    pub fn load() -> Result<Self> {
        if let Ok(env) = std::env::var("QWERT_PROFILE") {
            let profile = if env.trim().is_empty() {
                None
            } else {
                Some(env.trim().to_string())
            };
            return Ok(Self { profile });
        }
        Self::load_from(&machine_path())
    }

    /// The active profile name, or "default" when not configured.
    pub fn active_profile(&self) -> &str {
        self.profile.as_deref().unwrap_or(super::qwert_yml::PROFILE_DEFAULT)
    }

    pub fn save(&self) -> Result<()> {
        self.save_to(&machine_path())
    }

    pub fn set_profile(&mut self, profile: String) {
        self.profile = Some(profile);
    }

    /// Parse a profile name from env override.
    pub fn from_env(env: &str) -> Self {
        let profile = if env.trim().is_empty() {
            None
        } else {
            Some(env.trim().to_string())
        };
        Self { profile }
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