use anyhow::Result;

use crate::config::qwert_yml;
use crate::recipe::index;
use crate::ui::printer;

pub fn run(name: &str, uninstall: bool) -> Result<()> {
    let manifest_path = qwert_yml::manifest_path();
    let mut config = qwert_yml::QwertConfig::load(&manifest_path)?;

    if !config.has_tool(name) {
        printer::warning(&format!("{} is not declared in qwert.yml", name));
        return Ok(());
    }

    config.remove_tool(name);
    config.save(&manifest_path)?;
    printer::ok(name, "removed from qwert.yml");

    if uninstall {
        let recipes_dir = index::cache_dir()
            .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;

        match index::find(name, &recipes_dir) {
            Some(_recipe) => {
                // Uninstall logic will be implemented per-recipe
                // For now, inform the user
                printer::info(&format!(
                    "To uninstall manually: brew uninstall {}",
                    name
                ));
            }
            None => {
                printer::warning(&format!("recipe '{}' not found — uninstall manually", name));
            }
        }
    }

    Ok(())
}
