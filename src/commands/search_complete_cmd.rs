use anyhow::Result;
use std::collections::HashSet;

use crate::platform::{self, Platform};
use crate::recipe::index;

/// Output `name\tdescription` per line — consumed by shell completions.
/// Searches local recipes first, then brew, excluding recipe duplicates.
pub fn run(term: &str) -> Result<()> {
    if term.len() < 2 {
        return Ok(());
    }

    let recipes_dir = index::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;

    let q = term.to_lowercase();

    let qwert_results: Vec<_> = index::load_all(&recipes_dir)
        .into_iter()
        .filter(|r| r.meta.name.to_lowercase().contains(&q) || r.meta.description.to_lowercase().contains(&q))
        .collect();

    let qwert_names: HashSet<String> = qwert_results.iter().map(|r| r.meta.name.clone()).collect();

    for recipe in &qwert_results {
        println!("{}\t{}", recipe.meta.name, recipe.meta.description);
    }

    if platform::detect() == Platform::MacOS {
        for name in brew_search(term, &qwert_names) {
            println!("{}\t", name);
        }
    }

    Ok(())
}

fn brew_search(term: &str, exclude: &HashSet<String>) -> Vec<String> {
    let output = std::process::Command::new("brew")
        .args(["search", term])
        .output();

    let Ok(out) = output else { return vec![] };
    if !out.status.success() { return vec![]; }

    String::from_utf8_lossy(&out.stdout)
        .lines()
        .flat_map(|l| l.split_whitespace())
        .map(|s| s.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('=') && !exclude.contains(l))
        .collect()
}
