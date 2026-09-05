use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Machine-local identity: which profile and which platform this machine runs.
/// A machine runs exactly one profile and one platform. Stored in
/// ~/.local/share/qwert/machine.yml; overridden by QWERT_PROFILE and QWERT_PLATFORM env vars.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct MachineIdentity {
    /// The active profile for this machine. None = not configured yet.
    #[serde(default)]
    pub profile: Option<String>,
    /// Explicit platform override (macos|debian|arch). None = auto-detect.
    #[serde(default)]
    pub platform: Option<String>,
}

impl MachineIdentity {
    /// Load from env overrides (QWERT_PROFILE / QWERT_PLATFORM) or machine.yml.
    pub fn load() -> Result<Self> {
        let env_profile = std::env::var("QWERT_PROFILE").ok();
        let env_platform = std::env::var("QWERT_PLATFORM").ok();
        if env_profile.is_some() || env_platform.is_some() {
            return Ok(Self::from_env(env_profile.as_deref(), env_platform.as_deref()));
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

    pub fn set_platform(&mut self, platform: String) {
        self.platform = Some(platform);
    }

    /// Parse profile + platform from env overrides.
    fn from_env(profile: Option<&str>, platform: Option<&str>) -> Self {
        let profile = profile.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        let platform = platform.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        Self { profile, platform }
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