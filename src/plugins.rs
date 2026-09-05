use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::qwert_yml::{QwertConfig, manifest_path};
use crate::ui::printer;

/// The default recipes catalog, always cloned into the runtime cache.
pub const DEFAULT_RECIPES_URL: &str = "https://github.com/br4zz4/qwert-recipes";

/// Plugin info for `qwert plugin list`.
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub url: String,
    pub cloned: bool,
}

/// Runtime directory holding plugin clones: ~/.local/share/qwert/plugins/
pub fn dir() -> PathBuf {
    crate::platform::data_dir().join("plugins")
}

/// Run git with silent stream capture; fails with stderr on non-zero exit.
fn git(args: &[&str]) -> Result<()> {
    let out = Command::new("git")
        .args(args)
        .output()
        .context("failed to run git")?;
    if out.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        anyhow::bail!("git {} failed: {}", args.join(" "), stderr)
    }
}

/// Derive a plugin name from its URL: last path segment, `.git` stripped,
/// accepting only alphanumerics, `-` and `_` (like asdf's name from repo).
pub fn derive_name(url: &str) -> Result<String> {
    let trimmed = url.trim().trim_end_matches('/');
    let trimmed = trimmed.strip_suffix(".git").unwrap_or(trimmed);
    let last = trimmed.rsplit('/').next().unwrap_or("").trim();
    if last.is_empty() {
        anyhow::bail!("cannot derive plugin name from url: {}", url);
    }
    if !last.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        anyhow::bail!(
            "plugin name '{}' contains invalid characters — name a repo with letters, digits, '-' or '_'",
            last
        );
    }
    Ok(last.to_string())
}

/// Clone directory for a plugin name.
pub fn clone_dir(name: &str) -> PathBuf {
    dir().join(name)
}

/// Clone a plugin (or the default catalog) into `dest`. No-op if already present.
/// A pre-existing non-git directory (e.g. the old tarball cache) is cleared first —
/// only within qwert's own data dir, so arbitrary paths are never touched.
pub fn clone(url: &str, dest: &Path) -> Result<()> {
    if dest.join(".git").is_dir() {
        return Ok(());
    }
    if dest.is_dir() {
        let data = crate::platform::data_dir();
        if !dest.starts_with(&data) {
            anyhow::bail!(
                "refusing to clear non-qwert directory: {} — remove it manually",
                dest.display()
            );
        }
        std::fs::remove_dir_all(dest)
            .with_context(|| format!("failed to clear stale cache {}", dest.display()))?;
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    git(&["clone", "--depth", "1", url, &dest.to_string_lossy()])
}

/// Pull latest changes for an existing plugin clone (fast-forward only).
pub fn pull(dest: &Path) -> Result<()> {
    if !dest.join(".git").is_dir() {
        anyhow::bail!("not a git clone: {}", dest.display());
    }
    git(&["-C", &dest.to_string_lossy(), "pull", "--ff-only"])
}

/// Declare a plugin by URL: derive name, clone it, then record it in config.yml.
pub fn add(url: &str) -> Result<()> {
    let name = derive_name(url)?;
    let mut config = QwertConfig::load(&manifest_path())?;

    if let Some(existing) = config.plugins().iter().find(|p| p.name == name) {
        if existing.url != url {
            anyhow::bail!(
                "plugin '{}' is already declared with a different url: {}",
                name,
                existing.url
            );
        }
    }

    printer::installing("git", &format!("cloning {}", url));
    clone(url, &clone_dir(&name))?;
    config.add_plugin(&name, url);
    config.save(&manifest_path())?;
    printer::ok("plugin", &format!("{} added ({})", name, url));
    printer::blank();
    Ok(())
}

/// Remove a plugin: drop its clone and remove the declaration from config.yml.
pub fn remove(name: &str) -> Result<()> {
    let mut config = QwertConfig::load(&manifest_path())?;
    if !config.remove_plugin(name) {
        anyhow::bail!("plugin '{}' is not declared", name);
    }
    config.save(&manifest_path())?;

    let dir = clone_dir(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).context("failed to remove plugin clone")?;
    }
    printer::ok("plugin", &format!("{} removed", name));
    printer::blank();
    Ok(())
}

/// List declared plugins and whether each one is cloned.
pub fn list() -> Result<Vec<PluginInfo>> {
    let config = QwertConfig::load(&manifest_path())?;
    Ok(config
        .plugins()
        .iter()
        .map(|p| PluginInfo {
            name: p.name.clone(),
            url: p.url.clone(),
            cloned: clone_dir(&p.name).join(".git").is_dir(),
        })
        .collect())
}

/// Update every declared plugin: clone if missing, otherwise pull.
pub fn update_all() -> Result<()> {
    for plugin in list()? {
        if plugin.cloned {
            printer::installing("git", &format!("updating {}", plugin.name));
            pull(&clone_dir(&plugin.name))?;
            printer::ok("plugin", &format!("{} updated", plugin.name));
        } else {
            clone(&plugin.url, &clone_dir(&plugin.name))?;
        }
    }
    printer::blank();
    Ok(())
}

/// Ensure every plugin declared in config.yml is cloned. Used before recipe lookup
/// so a replicated ~/.qwert (new machine) restores its plugins on the first command.
pub fn ensure_clones() -> Result<()> {
    for plugin in list()? {
        if !plugin.cloned {
            clone(&plugin.url, &clone_dir(&plugin.name))?;
        }
    }
    Ok(())
}

/// Recipe search directories for declared plugins, in declaration order.
/// Only returns directories that actually have a clone.
pub fn recipe_dirs() -> Vec<PathBuf> {
    let Ok(config) = QwertConfig::load(&manifest_path()) else {
        return Vec::new();
    };
    config
        .plugins()
        .iter()
        .map(|p| clone_dir(&p.name).join("recipes"))
        .filter(|p| p.is_dir())
        .collect()
}

#[cfg(test)]
#[path = "tests/plugins.rs"]
mod tests;