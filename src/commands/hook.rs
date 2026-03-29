use anyhow::Result;

use crate::config::qwert_yml;

pub fn run(phase: &str) -> Result<()> {
    let manifest_path = qwert_yml::manifest_path();
    let config = qwert_yml::QwertConfig::load(&manifest_path)?;

    let scripts = match phase {
        "init" => &config.scripts.init,
        "end" => &config.scripts.end,
        _ => return Ok(()),
    };

    for path in scripts {
        let expanded = qwert_yml::expand_tilde(path);
        println!("[ -f \"{}\" ] && source \"{}\"", expanded, expanded);
    }

    Ok(())
}
