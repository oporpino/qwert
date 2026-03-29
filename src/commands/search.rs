use anyhow::Result;

use crate::recipe::index;
use crate::ui::printer;

pub fn run(term: Option<&str>) -> Result<()> {
    let recipes_dir = index::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;

    let all = index::load_all(&recipes_dir);

    if all.is_empty() {
        printer::info("No recipes found. Run `qwert update` to fetch the latest recipes.");
        return Ok(());
    }

    let results: Vec<_> = if let Some(q) = term {
        let q = q.to_lowercase();
        all.into_iter()
            .filter(|r| {
                r.meta.name.to_lowercase().contains(&q)
                    || r.meta.description.to_lowercase().contains(&q)
            })
            .collect()
    } else {
        all
    };

    if results.is_empty() {
        printer::info("No recipes matched your search.");
        return Ok(());
    }

    printer::blank();
    for recipe in &results {
        printer::search_result(
            &recipe.meta.name,
            &recipe.meta.kind.to_string(),
            &recipe.meta.description,
            Some(&recipe.meta.version),
        );
    }
    printer::blank();

    Ok(())
}
