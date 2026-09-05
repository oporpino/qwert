use anyhow::Result;
use std::collections::HashSet;

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

    // Search the platform's package manager via yuiop
    let yuiop_results = crate::adapters::yuiop::search(term)
        .into_iter()
        .filter(|n| !qwert_names.contains(n))
        .collect::<Vec<_>>();

    if qwert_results.is_empty() && yuiop_results.is_empty() {
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

    for name in &yuiop_results {
        printer::search_result(name, "yuiop", "", None);
    }

    printer::blank();
    Ok(())
}