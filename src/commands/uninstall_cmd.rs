use anyhow::Result;

use crate::config::qwert_yml;
use crate::recipe::{index, runner};
use crate::ui::printer;

pub fn run(name: &str) -> Result<()> {
    let manifest_path = qwert_yml::manifest_path();
    let mut config = qwert_yml::QwertConfig::load(&manifest_path)?;

    if !config.has_tool(name) {
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
            match crate::adapters::default_adapter() {
                Some(adapter) => {
                    if let Err(e) = crate::platform::run_cmd(&adapter.uninstall_cmd(name)) {
                        printer::failed(name, &e.to_string());
                    } else {
                        printer::ok(name, "uninstalled");
                    }
                }
                None => printer::failed(name, "no package manager available on this platform"),
            }
        }
    }

    Ok(())
}
