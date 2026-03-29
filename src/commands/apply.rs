use anyhow::Result;

use crate::config::qwert_yml;
use crate::recipe::{index, runner};
use crate::ui::printer;

pub fn run(tool: Option<&str>, dry_run: bool) -> Result<()> {
    let manifest_path = qwert_yml::manifest_path();
    let config = qwert_yml::QwertConfig::load(&manifest_path)?;

    let recipes_dir = index::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;

    let tools: Vec<&str> = if let Some(t) = tool {
        vec![t]
    } else {
        config.tools.iter().map(|s| s.as_str()).collect()
    };

    if tools.is_empty() {
        printer::info("No tools declared. Run `qwert use <tool>` to add one.");
        return Ok(());
    }

    printer::h1("Applying machine setup...");
    printer::blank();

    let mut done = 0;
    let mut failed = 0;

    for name in &tools {
        if dry_run {
            printer::bullet(&format!("would install: {}", name));
            continue;
        }

        match index::find(name, &recipes_dir) {
            Some(recipe) => {
                printer::installing(name, "...");
                if runner::install_with_output(&recipe) {
                    done += 1;
                } else {
                    failed += 1;
                }
            }
            None => {
                printer::failed(name, "recipe not found");
                failed += 1;
            }
        }
    }

    if !dry_run {
        printer::summary(done, tools.len(), failed);
    }

    Ok(())
}
