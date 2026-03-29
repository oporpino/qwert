use anyhow::Result;

use crate::config::qwert_yml;
use crate::recipe::{index, runner};
use crate::ui::printer;

pub fn run() -> Result<()> {
    let manifest_path = qwert_yml::manifest_path();
    let config = qwert_yml::QwertConfig::load(&manifest_path)?;

    if config.tools.is_empty() {
        printer::info("No tools declared. Run `qwert use <tool>` to add one.");
        return Ok(());
    }

    let recipes_dir = index::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;

    printer::blank();

    for name in &config.tools {
        match index::find(name, &recipes_dir) {
            Some(recipe) => runner::status_with_output(&recipe),
            None => printer::failed(name, "recipe not found"),
        }
    }

    if !config.stacks.is_empty() {
        printer::h2("Stacks");
        for stack in &config.stacks {
            printer::bullet(stack);
        }
    }

    printer::blank();
    Ok(())
}
