use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::platform::fs;

/// Materialize a merged view of `~/.qwert/<tool>/` plus per-role overrides.
///
/// Returns `Ok(None)` when no active role has an `overrides/<role>/` directory —
/// in that case the caller should use `~/.qwert/<tool>/` directly (no merge needed).
///
/// Otherwise recreates `~/.local/share/qwert/merged/<tool>/` by copying the base
/// tree (excluding `overrides/`) and then applying each active role's override in
/// order — later roles overwrite earlier files ("last wins").
pub fn materialize(
    tool: &str,
    roles: &[String],
    config_dir: &Path,
    data_dir: &Path,
) -> Result<Option<PathBuf>> {
    let base = config_dir.join("config").join(tool);
    if !base.is_dir() {
        return Ok(None);
    }

    let has_overrides = roles
        .iter()
        .any(|r| base.join("overrides").join(r).is_dir());
    if !has_overrides {
        return Ok(None);
    }

    let merged = data_dir.join("merged").join(tool);
    if merged.exists() {
        std::fs::remove_dir_all(&merged)?;
    }
    std::fs::create_dir_all(&merged)?;

    // Base tree (skip the overrides/ container itself).
    fs::copy_dir_excluding(&base, &merged, Some("overrides"))?;

    // Apply each active role in order (last wins).
    for role in roles {
        let override_dir = base.join("overrides").join(role);
        if override_dir.is_dir() {
            fs::copy_dir(&override_dir, &merged)?;
        }
    }

    Ok(Some(merged))
}

#[cfg(test)]
#[path = "tests/merge.rs"]
mod tests;
