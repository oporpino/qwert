use anyhow::Result;
use std::collections::HashSet;

use crate::platform::{self, Platform};
use crate::recipe::index;
use crate::ui::printer;

pub fn run(term: &str) -> Result<()> {
    let recipes_dir = index::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;

    let q = term.to_lowercase();

    // Search qwert recipes
    let qwert_results: Vec<_> = index::load_all(&recipes_dir)
        .into_iter()
        .filter(|r| r.meta.name.to_lowercase().contains(&q) || r.meta.description.to_lowercase().contains(&q))
        .collect();

    let qwert_names: HashSet<String> = qwert_results.iter().map(|r| r.meta.name.clone()).collect();

    // Search brew (macOS only)
    let brew_results = if platform::detect() == Platform::MacOS {
        brew_search(term, &qwert_names)
    } else {
        vec![]
    };

    if qwert_results.is_empty() && brew_results.is_empty() {
        printer::info(&format!("No results for \"{}\".", term));
        return Ok(());
    }

    printer::blank();

    for recipe in &qwert_results {
        printer::search_result(
            &recipe.meta.name,
            &recipe.meta.kind.to_string(),
            &recipe.meta.description,
            None,
        );
    }

    for name in &brew_results {
        printer::search_result(name, "brew", "", None);
    }

    printer::blank();
    Ok(())
}

/// Run `brew search` and return results not already in qwert recipes.
fn brew_search(term: &str, exclude: &HashSet<String>) -> Vec<String> {
    let output = std::process::Command::new("brew")
        .args(["search", term])
        .output();

    let Ok(out) = output else { return vec![] };
    if !out.status.success() { return vec![]; }

    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('=') && !exclude.contains(l))
        .collect()
}
