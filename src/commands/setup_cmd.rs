use anyhow::Result;

use crate::config::qwert_yml;
use crate::recipe::{index, runner};
use crate::ui::printer;

fn active_profile() -> String {
    crate::config::machine::MachineIdentity::load()
        .map(|m| m.active_profile().to_string())
        .unwrap_or_else(|_| qwert_yml::PROFILE_DEFAULT.to_string())
}

pub fn run(name: &str) -> Result<()> {
    let manifest_path = qwert_yml::manifest_path();
    let mut config = qwert_yml::QwertConfig::load(&manifest_path)?;
    let profile = active_profile();

    if !config.has_tool_in(&profile, name) {
        config.add_tool(&profile, name, None);
        config.save(&manifest_path)?;
        printer::ok(name, &format!("added to qwert.yml ({})", profile));
    }

    let recipes_dir = index::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;

    crate::commands::recipes_cmd::update_silent();
    crate::plugins::ensure_clones().ok();

    let source = config
        .config_source_for(&profile, name)
        .map(|s| std::path::PathBuf::from(qwert_yml::expand_tilde(s)));

    let recipe = index::find(name, &recipes_dir);
    let recipe_has_setup = recipe.as_ref().map(|r| r.setup.is_some()).unwrap_or(false);

    if recipe_has_setup {
        runner::setup_with_output(recipe.as_ref().unwrap(), source.as_deref());
    } else if let Some(inline) = config.inline_setup_of(name) {
        runner::setup_inline_with_output(name, inline, source.as_deref());
    } else {
        printer::warning(&format!("no setup defined for '{}' — nothing to setup", name));
    }

    Ok(())
}