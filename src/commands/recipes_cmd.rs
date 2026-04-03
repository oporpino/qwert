use anyhow::{Context, Result};
use std::process::Command;

use crate::recipe::index::cache_dir;
use crate::ui::printer;

const TARBALL_URL: &str =
    "https://github.com/oporpino/qwert/archive/refs/heads/main.tar.gz";

const VERSION_URL: &str =
    "https://raw.githubusercontent.com/oporpino/qwert/main/recipes/VERSION";

pub fn update() -> Result<()> {
    printer::h1("Updating recipes...");
    printer::blank();
    fetch()?;
    printer::ok("recipes", "updated");
    printer::blank();
    Ok(())
}

/// Silent best-effort update — fetches only when remote VERSION differs from cache.
/// Errors are ignored so offline usage is unaffected.
pub fn update_silent() {
    let remote = match fetch_version() {
        Some(v) => v,
        None => return,
    };

    let local = cache_dir()
        .and_then(|c| std::fs::read_to_string(c.join("VERSION")).ok())
        .unwrap_or_default();

    if local.trim() != remote.trim() {
        let _ = fetch();
    }
}

fn fetch_version() -> Option<String> {
    let output = Command::new("curl")
        .args(["-fsSL", "--max-time", "5", VERSION_URL])
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn fetch() -> Result<()> {
    let cache = cache_dir().context("cannot determine home dir")?;
    std::fs::create_dir_all(&cache)?;

    let tmp = std::env::temp_dir().join("qwert-recipes-tmp");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp)?;

    let status = Command::new("sh")
        .arg("-c")
        .arg(format!("curl -fsSL '{TARBALL_URL}' | tar -xz -C '{}'", tmp.display()))
        .status()
        .context("failed to run curl/tar")?;

    if !status.success() {
        anyhow::bail!("failed to download recipes tarball");
    }

    // Find extracted dir (e.g. qwert-main or qwert-<sha>)
    let extracted = std::fs::read_dir(&tmp)?
        .filter_map(|e| e.ok())
        .find(|e| e.path().is_dir())
        .context("no directory found in tarball")?;

    let src = extracted.path().join("recipes");
    if !src.exists() {
        anyhow::bail!("no recipes/ directory in tarball");
    }

    let _ = std::fs::remove_dir_all(&cache);
    std::fs::create_dir_all(&cache)?;

    for entry in std::fs::read_dir(&src)?.filter_map(|e| e.ok()) {
        let src_path = entry.path();
        let dst_path = cache.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    std::fs::remove_dir_all(&tmp)?;
    Ok(())
}

fn copy_dir(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)?.filter_map(|e| e.ok()) {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
