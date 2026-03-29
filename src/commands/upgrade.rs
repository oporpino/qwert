use anyhow::Result;

use crate::config::qwert_yml;
use crate::recipe::{index, runner};
use crate::ui::printer;

pub fn run(tool: Option<&str>) -> Result<()> {
    let manifest_path = qwert_yml::manifest_path();
    let config = qwert_yml::QwertConfig::load(&manifest_path)?;

    let recipes_dir = index::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;

    let tool_names = config.tool_names();
    let tools: Vec<&str> = if let Some(t) = tool {
        vec![t]
    } else {
        tool_names.iter().map(|s| s.as_str()).collect()
    };

    if tools.is_empty() {
        printer::info("No tools declared.");
        return Ok(());
    }

    printer::h1("Upgrading tools...");
    printer::blank();

    for name in &tools {
        match index::find(name, &recipes_dir) {
            Some(recipe) => {
                let result = runner::upgrade(&recipe);
                match result {
                    runner::RunResult::Installed => {
                        let version = runner::installed_version(&recipe);
                        let msg = version
                            .map(|v| format!("upgraded ({})", v))
                            .unwrap_or_else(|| "upgraded".to_string());
                        printer::ok(name, &msg);
                    }
                    runner::RunResult::NotSupported => {
                        printer::ok(name, "no upgrade available");
                    }
                    runner::RunResult::Failed(err) => {
                        printer::failed(name, &err);
                    }
                    runner::RunResult::AlreadyInstalled { .. } => {
                        printer::ok(name, "already up to date");
                    }
                }
            }
            None => printer::failed(name, "recipe not found"),
        }
    }

    printer::blank();
    Ok(())
}
