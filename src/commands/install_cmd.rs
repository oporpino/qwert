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

    match index::find(name, &recipes_dir) {
        Some(recipe) => {
            runner::install_with_output(&recipe, &recipes_dir);
        }
        None => {
            if crate::platform::which(name) {
                printer::ok(name, "already installed");
            } else {
                match crate::adapters::yuiop::install(name) {
                    Ok(_) => printer::ok(name, "installed"),
                    Err(e) => printer::failed(name, &e),
                }
            }
        }
    }

    Ok(())
}