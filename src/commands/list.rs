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
    let config_dir = qwert_yml::config_dir();

    printer::blank();

    for name in config.tool_names() {
        let declared = config.version_of(&name);
        match index::find(&name, &recipes_dir) {
            Some(recipe) => runner::status_with_setup_output(&recipe, &config_dir, declared),
            None => {
                let installed = crate::platform::which(&name);
                let tag = printer::kind_tag_col("—");
                let install_str = if installed { "installed" } else { "not installed" };
                let msg = format!("{:<28}{}  {:<12}  {}", install_str, tag, "—", declared);
                if installed { printer::ok(&name, &msg); } else { printer::failed(&name, &msg); }
            }
        }
    }

    printer::blank();
    Ok(())
}
