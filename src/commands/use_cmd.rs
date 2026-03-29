use anyhow::Result;

use crate::config::qwert_yml;
use crate::recipe::{index, runner};
use crate::ui::printer;

pub fn use_tool(name: &str, no_install: bool) -> Result<()> {
    let manifest_path = qwert_yml::manifest_path();
    let mut config = qwert_yml::QwertConfig::load(&manifest_path)?;

    if config.has_tool(name) {
        printer::info(&format!("{} is already declared in qwert.yml", name));
    } else {
        config.add_tool(name);
        config.save(&manifest_path)?;
        printer::ok(name, "added to qwert.yml");
    }

    if !no_install {
        let recipes_dir = index::cache_dir()
            .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;

        match index::find(name, &recipes_dir) {
            Some(recipe) => {
                runner::install_with_output(&recipe, &recipes_dir);
            }
            None => {
                printer::warning(&format!(
                    "recipe '{}' not found — added to qwert.yml but not installed",
                    name
                ));
            }
        }
    }

    Ok(())
}

pub fn use_script(hook: &str, path: &str) -> Result<()> {
    let manifest_path = qwert_yml::manifest_path();
    let mut config = qwert_yml::QwertConfig::load(&manifest_path)?;

    config.add_script(hook, path);
    config.save(&manifest_path)?;

    printer::ok(
        "script",
        &format!("added to {} hook in qwert.yml", hook),
    );
    printer::info("Restart your shell or run `source ~/.zshrc` to apply.");

    Ok(())
}
