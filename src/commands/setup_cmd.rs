use anyhow::Result;

use crate::config::qwert_yml;
use crate::recipe::{index, runner};
use crate::ui::printer;

pub fn run(name: &str) -> Result<()> {
    let manifest_path = qwert_yml::manifest_path();
    let mut config = qwert_yml::QwertConfig::load(&manifest_path)?;

    if !config.has_tool(name) {
        config.add_tool(name, None);
        config.save(&manifest_path)?;
        printer::ok(name, "added to qwert.yml");
    }

    let recipes_dir = index::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;
    let config_dir = qwert_yml::config_dir();

    crate::commands::recipes_cmd::update_silent();

    let recipe = index::find(name, &recipes_dir);
    let recipe_has_setup = recipe.as_ref().map(|r| r.setup.is_some()).unwrap_or(false);

    if recipe_has_setup {
        runner::setup_with_output(recipe.as_ref().unwrap(), &config_dir);
    } else if let Some(inline) = config.setup_of(name) {
        runner::setup_inline_with_output(name, inline, &config_dir);
    } else {
        printer::warning(&format!("no setup defined for '{}' — nothing to setup", name));
    }

    Ok(())
}
