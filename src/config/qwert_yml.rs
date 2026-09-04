use anyhow::{Context, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Deserializer, Serialize};
use std::path::{Path, PathBuf};

/// Profile name used when a legacy config (top-level `tools:`) is loaded.
pub const PROFILE_DEFAULT: &str = "default";

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

/// Inline setup defined in config.yml — mirrors RecipeSetup without importing recipe module.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InlineSetup {
    pub from: Option<String>,
    pub to: String,
    #[serde(default)]
    pub symlink: bool,
    pub macos: Option<StringOrList>,
    pub debian: Option<StringOrList>,
    pub arch: Option<StringOrList>,
    pub undo: Option<InlineUndo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InlineUndo {
    pub macos: Option<StringOrList>,
    pub debian: Option<StringOrList>,
    pub arch: Option<StringOrList>,
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

/// Hooks for a single profile.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Hooks {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prepare: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub init: Vec<String>,
}

/// A single profile: its own list of tools and hooks.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Profile {
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub tools: IndexMap<String, ToolEntry>,

    #[serde(default, skip_serializing_if = "hooks_empty")]
    pub hooks: Hooks,
}

fn hooks_empty(h: &Hooks) -> bool {
    h.prepare.is_empty() && h.init.is_empty()
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(from = "QwertConfigRaw", into = "QwertConfigRaw")]
pub struct QwertConfig {
    /// profile name → profile (tools + hooks).
    pub profiles: IndexMap<String, Profile>,
}

/// Raw shape used for (de)serialization — supports both legacy flat (top-level
/// `tools:`/`hooks:`) and the new `profiles:` form.
#[derive(Debug, Default, Deserialize, Serialize)]
struct QwertConfigRaw {
    #[serde(default)]
    profiles: IndexMap<String, Profile>,

    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    tools: IndexMap<String, ToolEntry>,

    #[serde(default, skip_serializing_if = "hooks_empty")]
    hooks: Hooks,
}

impl From<QwertConfigRaw> for QwertConfig {
    fn from(raw: QwertConfigRaw) -> Self {
        if !raw.profiles.is_empty() {
            QwertConfig { profiles: raw.profiles }
        } else {
            // Legacy flat config → single "default" profile.
            let mut profiles = IndexMap::new();
            profiles.insert(
                PROFILE_DEFAULT.to_string(),
                Profile { tools: raw.tools, hooks: raw.hooks },
            );
            QwertConfig { profiles }
        }
    }
}

impl From<QwertConfig> for QwertConfigRaw {
    fn from(cfg: QwertConfig) -> Self {
        QwertConfigRaw { profiles: cfg.profiles, tools: IndexMap::new(), hooks: Hooks::default() }
    }
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

    /// Names of all profiles.
    pub fn profile_names(&self) -> Vec<String> {
        self.profiles.keys().cloned().collect()
    }

    /// Does a profile exist (even with no tools)?
    pub fn has_profile(&self, profile: &str) -> bool {
        self.profiles.contains_key(profile)
    }

    /// Names of profiles that declare at least one tool.
    pub fn profiles_with_tools(&self) -> Vec<String> {
        self.profiles
            .iter()
            .filter(|(_, p)| !p.tools.is_empty())
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// Get (or create) the profile.
    pub fn ensure_profile(&mut self, profile: &str) -> &mut Profile {
        self.profiles.entry(profile.to_string()).or_default()
    }

    /// Is a tool declared in any profile?
    pub fn declared_anywhere(&self, name: &str) -> bool {
        self.profiles.values().any(|p| p.tools.contains_key(name))
    }

    /// Is a tool declared in a specific profile?
    pub fn has_tool_in(&self, profile: &str, name: &str) -> bool {
        self.profiles
            .get(profile)
            .map(|p| p.tools.contains_key(name))
            .unwrap_or(false)
    }

    /// Add or update a tool in a profile. `version` defaults to "latest" if None.
    /// Preserves existing inline setup when updating an existing entry.
    pub fn add_tool(&mut self, profile: &str, name: &str, version: Option<&str>) {
        let ver = version.unwrap_or("latest").to_string();
        self.ensure_profile(profile)
            .tools
            .entry(name.to_string())
            .and_modify(|e| match e {
                ToolEntry::Simple(v) => *v = ver.clone(),
                ToolEntry::Full(c) => c.version = ver.clone(),
            })
            .or_insert_with(|| ToolEntry::Simple(ver));
    }

    /// Remove a tool from every profile; drop profiles that become empty.
    pub fn remove_tool(&mut self, name: &str) {
        for profile in self.profiles.values_mut() {
            profile.tools.shift_remove(name);
        }
        self.profiles.retain(|_, p| !p.tools.is_empty() || !hooks_empty(&p.hooks));
    }

    /// Tools declared for the given profile, in declaration order.
    pub fn tool_names_for_profile(&self, profile: &str) -> Vec<String> {
        match self.profiles.get(profile) {
            Some(p) => p.tools.keys().cloned().collect(),
            None => Vec::new(),
        }
    }

    /// Version for a tool in a profile. Defaults to "latest".
    pub fn version_of(&self, profile: &str, name: &str) -> &str {
        match self.profiles.get(profile).and_then(|p| p.tools.get(name)) {
            Some(ToolEntry::Simple(v)) => v.as_str(),
            Some(ToolEntry::Full(c)) => c.version.as_str(),
            None => "latest",
        }
    }

    /// Inline setup for a tool declared in a profile.
    pub fn setup_of(&self, profile: &str, name: &str) -> Option<&InlineSetup> {
        match self.profiles.get(profile).and_then(|p| p.tools.get(name)) {
            Some(ToolEntry::Full(c)) => c.setup.as_ref(),
            _ => None,
        }
    }

    /// The profile that declares this tool (first match), if any.
    pub fn profile_of_tool(&self, name: &str) -> Option<&str> {
        self.profiles
            .iter()
            .find(|(_, p)| p.tools.contains_key(name))
            .map(|(k, _)| k.as_str())
    }

    /// List of profiles that declare this tool (for display).
    pub fn profiles_of_tool(&self, name: &str) -> Vec<String> {
        self.profiles
            .iter()
            .filter(|(_, p)| p.tools.contains_key(name))
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// Append a hook path to a profile's prepare/init list (dedup). No-op for unknown hooks.
    pub fn add_hook(&mut self, profile: &str, hook: &str, path: &str) {
        if hook != "prepare" && hook != "init" {
            return;
        }
        let hooks = &mut self.ensure_profile(profile).hooks;
        let scripts = match hook {
            "prepare" => &mut hooks.prepare,
            _ => &mut hooks.init,
        };
        if !scripts.iter().any(|s| s == path) {
            scripts.push(path.to_string());
        }
    }

    /// Hook paths for a profile's prepare or init phase.
    pub fn hooks_for(&self, profile: &str, phase: &str) -> Vec<String> {
        let Some(hooks) = self.profiles.get(profile).map(|p| &p.hooks) else {
            return Vec::new();
        };
        match phase {
            "prepare" => hooks.prepare.clone(),
            "init" => hooks.init.clone(),
            _ => Vec::new(),
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
#[path = "tests/qwert_yml.rs"]
mod tests;