use anyhow::Result;

use crate::config::qwert_yml;

pub fn run(phase: &str) -> Result<()> {
    if phase != "before" && phase != "init" {
        return Ok(());
    }

    // Auto-source recipe-generated fragments from ~/.qwert/hooks/{phase}/
    if let Some(home) = dirs::home_dir() {
        let hooks_dir = home.join(".qwert").join("hooks").join(phase);
        if hooks_dir.is_dir() {
            let mut entries: Vec<_> = std::fs::read_dir(&hooks_dir)
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path().extension().map(|x| x == "sh").unwrap_or(false)
                })
                .collect();
            entries.sort_by_key(|e| e.file_name());
            for entry in entries {
                let path = entry.path().to_string_lossy().to_string();
                println!("[ -f \"{}\" ] && source \"{}\"", path, path);
            }
        }
    }

    // User-defined hooks from qwert.yml
    let manifest_path = qwert_yml::manifest_path();
    let config = qwert_yml::QwertConfig::load(&manifest_path)?;

    let hooks = match phase {
        "before" => &config.hooks.before,
        "init" => &config.hooks.init,
        _ => return Ok(()),
    };

    for path in hooks {
        let expanded = qwert_yml::expand_tilde(path);
        println!("[ -f \"{}\" ] && source \"{}\"", expanded, expanded);
    }

    Ok(())
}
