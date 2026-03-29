use anyhow::Result;

use crate::config::qwert_yml;
use crate::recipe::{index, runner};
use crate::ui::printer;

pub fn run(name: &str) -> Result<()> {
    let manifest_path = qwert_yml::manifest_path();
    let mut config = qwert_yml::QwertConfig::load(&manifest_path)?;

    if !config.has_tool(name) {
        config.add_tool(name);
        config.save(&manifest_path)?;
        printer::ok(name, "added to qwert.yml");
    }

    let recipes_dir = index::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;
    let config_dir = qwert_yml::config_dir();

    crate::commands::recipes_cmd::update_silent();

    match index::find(name, &recipes_dir) {
        Some(recipe) => {
            runner::setup_with_output(&recipe, &config_dir);
        }
        None => {
            printer::warning(&format!("no recipe found for '{}' — nothing to setup", name));
        }
    }

    Ok(())
}
