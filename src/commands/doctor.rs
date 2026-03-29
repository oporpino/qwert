use anyhow::Result;

use crate::config::qwert_yml;
use crate::platform;
use crate::recipe::{index, runner};
use crate::ui::printer;

pub fn run() -> Result<()> {
    let manifest_path = qwert_yml::manifest_path();
    let config_dir = qwert_yml::config_dir();
    let recipes_dir = index::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;

    printer::h1("Doctor");
    printer::blank();

    // Platform
    let platform = platform::detect();
    printer::ok("platform", &platform.to_string());

    // Config dir
    if config_dir.exists() {
        printer::ok("config dir", &config_dir.display().to_string());
    } else {
        printer::failed("config dir", &format!("not found: {}", config_dir.display()));
    }

    // qwert.yml
    if manifest_path.exists() {
        printer::ok("qwert.yml", &manifest_path.display().to_string());
    } else {
        printer::failed("qwert.yml", "not found — run `qwert use <tool>` to create it");
    }

    // Recipes cache
    if recipes_dir.exists() {
        let count = std::fs::read_dir(&recipes_dir)
            .map(|d| d.count())
            .unwrap_or(0);
        printer::ok("recipes", &format!("{} cached in {}", count, recipes_dir.display()));
    } else {
        printer::failed("recipes", "cache not found — run `qwert update`");
    }

    // Tools status
    let config = qwert_yml::QwertConfig::load(&manifest_path)?;
    if !config.tools.is_empty() {
        printer::blank();
        printer::h2("Declared tools");
        for name in &config.tools {
            match index::find(name, &recipes_dir) {
                Some(recipe) => runner::status_with_output(&recipe),
                None => printer::failed(name, "recipe not found"),
            }
        }
    }

    printer::blank();
    Ok(())
}
