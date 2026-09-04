use anyhow::Result;

use crate::config::qwert_yml;

pub fn run(phase: &str) -> Result<()> {
    if phase != "prepare" && phase != "init" {
        return Ok(());
    }

    let config_dir = qwert_yml::config_dir();
    let machine_identity = crate::config::machine::MachineIdentity::load()?;
    let profile = machine_identity.active_profile().to_string();

    // Export the env vars hooks and configs rely on (QWERT_DIR, QWERT_PROFILE).
    println!("export QWERT_DIR=\"{}\"", config_dir.display());
    println!("export QWERT_PROFILE=\"{}\"", profile);

    // Auto-source recipe-generated fragments from ~/.local/share/qwert/hooks/{phase}/
    // (runtime artifacts; not user convention).
    {
        let hooks_dir = crate::platform::data_dir().join("hooks").join(phase);
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

    // User-defined hooks for the active profile, declared in config.yml.
    let manifest_path = qwert_yml::manifest_path();
    let config = qwert_yml::QwertConfig::load(&manifest_path)?;

    let hooks = config.hooks_for_profile(&profile, phase);
    for path in hooks {
        let expanded = qwert_yml::expand_tilde(&path);
        println!("[ -f \"{}\" ] && source \"{}\"", expanded, expanded);
    }

    Ok(())
}