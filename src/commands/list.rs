use anyhow::Result;

use crate::config::qwert_yml;
use crate::recipe::{index, runner};
use crate::ui::printer;

pub fn run() -> Result<()> {
    let manifest_path = qwert_yml::manifest_path();
    let config = qwert_yml::QwertConfig::load(&manifest_path)?;
    let profile = crate::config::machine::MachineIdentity::load()?.active_profile().to_string();

    let names = config.tool_names_for_profile(&profile);

    if names.is_empty() {
        printer::info("No tools declared. Run `qwert use <tool>` to add one.");
        return Ok(());
    }

    let recipes_dir = index::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;
    let config_dir = qwert_yml::config_dir();

    let name_width = names.iter().map(|n| n.len()).max().unwrap_or(0).max(12) + 2;

    printer::blank();
    printer::info(&format!("profile: {}", profile));
    printer::blank();

    for name in &names {
        let declared = config.version_of(&profile, name);
        let profiles = config.profiles_of_tool(name);
        let tag = if profiles.is_empty() {
            String::new()
        } else {
            format!(" [{}]", profiles.join(","))
        };
        let declared = format!("{}{}", declared, tag);
        match index::find(name, &recipes_dir) {
            Some(recipe) => runner::status_with_setup_output_w(&recipe, &config_dir, &declared, name_width),
            None => {
                let installed = crate::platform::which(name);
                let tag = printer::kind_tag_col("—");
                let install_str = if installed { "installed" } else { "not installed" };
                let msg = format!("{:<28}{}  {:<12}  {}", install_str, tag, "—", declared);
                if installed { printer::ok_w(name, name_width, &msg); } else { printer::failed_w(name, name_width, &msg); }
            }
        }
    }

    printer::blank();
    Ok(())
}