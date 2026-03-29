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
        printer::info("No tools declared. Run `qwert use <tool>` to add one.");
        return Ok(());
    }

    printer::blank();

    for name in &tools {
        match index::find(name, &recipes_dir) {
            Some(recipe) => runner::status_with_output(&recipe),
            None => printer::failed(name, "recipe not found"),
        }
    }

    printer::blank();
    Ok(())
}
