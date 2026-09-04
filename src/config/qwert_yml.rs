use anyhow::{Context, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Deserializer, Serialize};
use std::path::{Path, PathBuf};

/// The implicit, always-on role section. Base of the merge stack.
pub const SHARED: &str = "shared";

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

/// Hooks for a single role section.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct RoleHooks {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prepare: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub init: Vec<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct QwertConfig {
    /// role section → tool name → entry. "shared" is implicit.
    #[serde(
        default,
        deserialize_with = "deserialize_tools",
        serialize_with = "serialize_sections",
        skip_serializing_if = "sections_empty"
    )]
    pub tools: IndexMap<String, IndexMap<String, ToolEntry>>,

    /// role section → hooks. "shared" is implicit.
    #[serde(
        default,
        deserialize_with = "deserialize_hooks",
        serialize_with = "serialize_hook_sections",
        skip_serializing_if = "hook_sections_empty"
    )]
    pub hooks: IndexMap<String, RoleHooks>,
}

fn sections_empty(tools: &IndexMap<String, IndexMap<String, ToolEntry>>) -> bool {
    tools.is_empty()
}

fn hook_sections_empty(hooks: &IndexMap<String, RoleHooks>) -> bool {
    hooks.is_empty()
}

fn tool_config_keys() -> [&'static str; 2] {
    ["version", "setup"]
}

/// A value mapping is a "section" iff it has a key that is not a tool-config key.
fn is_tool_section(v: &serde_yml::Value) -> bool {
    match v.as_mapping() {
        Some(m) => m.iter().any(|(k, _)| match k.as_str() {
            Some(k) => !tool_config_keys().contains(&k),
            None => true,
        }),
        None => false,
    }
}

fn deserialize_tools<'de, D>(d: D) -> Result<IndexMap<String, IndexMap<String, ToolEntry>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_yml::Value::deserialize(d)?;
    let Some(map) = value.as_mapping() else {
        return Ok(IndexMap::new());
    };

    let all_sections = map.iter().all(|(_, v)| is_tool_section(v));

    if all_sections {
        let mut sections = IndexMap::new();
        for (k, v) in map {
            let section: IndexMap<String, ToolEntry> =
                serde_yml::from_value(v.clone()).map_err(serde::de::Error::custom)?;
            let name = k.as_str().map(|s| s.to_string()).unwrap_or_default();
            sections.insert(name, section);
        }
        Ok(sections)
    } else {
        let flat: IndexMap<String, ToolEntry> =
            serde_yml::from_value(serde_yml::Value::Mapping(map.clone()))
                .map_err(serde::de::Error::custom)?;
        let mut sections = IndexMap::new();
        sections.insert(SHARED.to_string(), flat);
        Ok(sections)
    }
}

/// Flat `hooks: {prepare, init}` (values are arrays) → shared.
/// Sectioned `hooks: {shared: {...}, dev: {...}}` (values are mappings) → sections.
fn deserialize_hooks<'de, D>(d: D) -> Result<IndexMap<String, RoleHooks>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_yml::Value::deserialize(d)?;
    let Some(map) = value.as_mapping() else {
        return Ok(IndexMap::new());
    };

    let all_sections = map.iter().all(|(_, v)| v.as_mapping().is_some());

    if all_sections {
        let mut sections = IndexMap::new();
        for (k, v) in map {
            let rh: RoleHooks = serde_yml::from_value(v.clone()).map_err(serde::de::Error::custom)?;
            let name = k.as_str().map(|s| s.to_string()).unwrap_or_default();
            sections.insert(name, rh);
        }
        Ok(sections)
    } else {
        let flat: RoleHooks = serde_yml::from_value(serde_yml::Value::Mapping(map.clone()))
            .map_err(serde::de::Error::custom)?;
        let mut sections = IndexMap::new();
        sections.insert(SHARED.to_string(), flat);
        Ok(sections)
    }
}

/// Serialize as nested sections (drops empty sections).
fn serialize_sections<S>(
    value: &IndexMap<String, IndexMap<String, ToolEntry>>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let filtered: IndexMap<_, _> = value
        .iter()
        .filter(|(_, tools)| !tools.is_empty())
        .collect();
    serde::Serialize::serialize(&filtered, s)
}

/// Serialize as nested sections (drops empty sections).
fn serialize_hook_sections<S>(value: &IndexMap<String, RoleHooks>, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let filtered: IndexMap<_, _> = value
        .iter()
        .filter(|(_, hooks)| !hooks.prepare.is_empty() || !hooks.init.is_empty())
        .collect();
    serde::Serialize::serialize(&filtered, s)
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

    /// Non-shared sections that have at least one tool declared.
    pub fn role_sections(&self) -> Vec<String> {
        self.tools
            .iter()
            .filter(|(k, v)| k.as_str() != SHARED && !v.is_empty())
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// Get (or create) the tool map for a role section.
    pub fn ensure_section(&mut self, role: &str) -> &mut IndexMap<String, ToolEntry> {
        self.tools.entry(role.to_string()).or_default()
    }

    /// Is a tool declared in any section?
    pub fn declared_anywhere(&self, name: &str) -> bool {
        self.tools.values().any(|s| s.contains_key(name))
    }

    /// Is a tool declared in a specific section?
    pub fn has_tool_in(&self, role: &str, name: &str) -> bool {
        self.tools
            .get(role)
            .map(|s| s.contains_key(name))
            .unwrap_or(false)
    }

    /// Add or update a tool in a role section. `version` defaults to "latest" if None.
    /// Preserves existing inline setup when updating an existing entry.
    pub fn add_tool(&mut self, name: &str, role: &str, version: Option<&str>) {
        let ver = version.unwrap_or("latest").to_string();
        self.ensure_section(role)
            .entry(name.to_string())
            .and_modify(|e| match e {
                ToolEntry::Simple(v) => *v = ver.clone(),
                ToolEntry::Full(c) => c.version = ver.clone(),
            })
            .or_insert_with(|| ToolEntry::Simple(ver));
    }

    /// Remove a tool from every section; drop sections that become empty.
    pub fn remove_tool(&mut self, name: &str) {
        for section in self.tools.values_mut() {
            section.shift_remove(name);
        }
        self.tools.retain(|_, s| !s.is_empty());
    }

    /// Ordered active sections: [shared] then roles in machine order (dedup, shared excluded).
    pub fn effective_sections(&self, roles: &[String]) -> Vec<String> {
        let mut v = vec![SHARED.to_string()];
        for r in roles {
            let r = r.trim();
            if !r.is_empty() && r != SHARED && !v.contains(&r.to_string()) {
                v.push(r.to_string());
            }
        }
        v
    }

    /// Union of tool names across active sections (first-seen order, dedup).
    pub fn tool_names_for_roles(&self, roles: &[String]) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        for section in self.effective_sections(roles) {
            if let Some(tools) = self.tools.get(&section) {
                for name in tools.keys() {
                    if !names.contains(name) {
                        names.push(name.clone());
                    }
                }
            }
        }
        names
    }

    /// Version for a tool across active sections — the last declaring section wins.
    pub fn version_of_for_roles(&self, name: &str, roles: &[String]) -> &str {
        for section in self.effective_sections(roles).iter().rev() {
            if let Some(entry) = self.tools.get(section).and_then(|s| s.get(name)) {
                return match entry {
                    ToolEntry::Simple(v) => v.as_str(),
                    ToolEntry::Full(c) => c.version.as_str(),
                };
            }
        }
        "latest"
    }

    /// Inline setup for a tool across active sections — the last declaring section wins.
    pub fn setup_of_for_roles(&self, name: &str, roles: &[String]) -> Option<&InlineSetup> {
        for section in self.effective_sections(roles).iter().rev() {
            if let Some(ToolEntry::Full(c)) = self.tools.get(section).and_then(|s| s.get(name)) {
                if let Some(setup) = &c.setup {
                    return Some(setup);
                }
            }
        }
        None
    }

    /// Sections that declare this tool (for display).
    pub fn sections_of_tool(&self, name: &str) -> Vec<String> {
        self.tools
            .iter()
            .filter(|(_, s)| s.contains_key(name))
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// Append a hook path to a role's prepare/init list (dedup). No-op for unknown hooks.
    pub fn add_hook(&mut self, role: &str, hook: &str, path: &str) {
        if hook != "prepare" && hook != "init" {
            return;
        }
        let rh = self.hooks.entry(role.to_string()).or_default();
        let scripts = match hook {
            "prepare" => &mut rh.prepare,
            _ => &mut rh.init,
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
#[path = "tests/qwert_yml.rs"]
mod tests;
