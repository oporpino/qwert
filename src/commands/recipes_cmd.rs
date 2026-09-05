use anyhow::{Context, Result};
use std::path::Path;

use crate::plugins;
use crate::recipe::index::cache_dir;
use crate::ui::printer;

/// Ensure the default recipes catalog is cloned and up to date (git, no tarballs).
fn sync_default(dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    if dir.join(".git").is_dir() {
        plugins::pull(dir)
    } else {
        plugins::clone(plugins::DEFAULT_RECIPES_URL, dir)
    }
}

pub fn update() -> Result<()> {
    printer::h1("Updating recipes...");
    printer::blank();

    let cache = cache_dir().context("cannot determine home dir")?;
    printer::installing("git", "syncing default recipes");
    sync_default(&cache)?;
    printer::ok("recipes", "updated");
    printer::blank();
    Ok(())
}

/// Silent best-effort update — pulls the default catalog only when it already
/// exists, cloning it on first use. Errors are ignored so offline usage is
/// unaffected.
pub fn update_silent() {
    let Some(cache) = cache_dir() else {
        return;
    };
    let _ = std::fs::create_dir_all(&cache);
    if cache.join(".git").is_dir() {
        let _ = plugins::pull(&cache);
    } else {
        let _ = plugins::clone(plugins::DEFAULT_RECIPES_URL, &cache);
    }
}