use anyhow::Result;

use crate::recipe::index;

/// Output `name\tdescription` per line — consumed by shell completions.
/// Searches only local recipes (no brew — too slow for interactive completion).
pub fn run(term: &str) -> Result<()> {
    if term.len() < 2 {
        return Ok(());
    }

    let recipes_dir = index::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;

    let q = term.to_lowercase();

    for recipe in index::load_all(&recipes_dir) {
        if recipe.meta.name.to_lowercase().contains(&q)
            || recipe.meta.description.to_lowercase().contains(&q)
        {
            println!("{}\t{}", recipe.meta.name, recipe.meta.description);
        }
    }

    Ok(())
}
