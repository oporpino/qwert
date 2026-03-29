use anyhow::Result;

use crate::config::qwert_yml;

pub fn run(phase: &str) -> Result<()> {
    let manifest_path = qwert_yml::manifest_path();
    let config = qwert_yml::QwertConfig::load(&manifest_path)?;

    let hooks = match phase {
        "init" => &config.hooks.init,
        "end" => &config.hooks.end,
        _ => return Ok(()),
    };

    for path in hooks {
        let expanded = qwert_yml::expand_tilde(path);
        println!("[ -f \"{}\" ] && source \"{}\"", expanded, expanded);
    }

    Ok(())
}
