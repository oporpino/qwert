use anyhow::Result;

use crate::config::qwert_yml;
use crate::recipe::{index, runner};
use crate::ui::printer;

pub fn run(name: &str) -> Result<()> {
    let manifest_path = qwert_yml::manifest_path();
    let mut config = qwert_yml::QwertConfig::load(&manifest_path)?;

    if !config.declared_anywhere(name) {
        printer::warning(&format!("{} is not declared in qwert.yml", name));
        return Ok(());
    }

    config.remove_tool(name);
    config.save(&manifest_path)?;
    printer::ok(name, "removed from qwert.yml");

    let recipes_dir = index::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;

    match index::find(name, &recipes_dir) {
        Some(recipe) => {
            runner::uninstall_with_output(&recipe);
        }
        None => {
            match crate::adapters::yuiop::uninstall(name) {
                Ok(_) => printer::ok(name, "uninstalled"),
                Err(e) => printer::failed(name, &e),
            }
        }
    }

    Ok(())
}
