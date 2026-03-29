use anyhow::Result;

use crate::recipe::{index, runner};
use crate::ui::printer;

pub fn run(name: &str) -> Result<()> {
    let recipes_dir = index::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;

    let recipe = match index::find(name, &recipes_dir) {
        Some(r) => r,
        None => {
            printer::failed(name, "recipe not found");
            return Ok(());
        }
    };

    printer::installing(name, "reinstalling...");

    // For now reinstall = upgrade (uninstall + install would require per-recipe logic)
    runner::install_with_output(&recipe);

    Ok(())
}
