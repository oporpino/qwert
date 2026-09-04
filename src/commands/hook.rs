use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::config::qwert_yml;

/// Recursively collect hook scripts under `dir` matching `<phase>.sh` and
/// `<phase>.<profile>.sh`, sorted by path (shared variant sorts before profile one).
fn collect_hook_scripts(dir: &Path, phase: &str, profile: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut paths: Vec<_> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();

    for path in paths {
        if path.is_dir() {
            collect_hook_scripts(&path, phase, profile, out);
            continue;
        }
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let shared = name == format!("{}.sh", phase);
        let profiled = name == format!("{}.{}.sh", phase, profile);
        if shared || profiled {
            out.push(path);
        }
    }
}

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

    // User-defined hooks for the active profile from qwert.yml.
    let manifest_path = qwert_yml::manifest_path();
    let config = qwert_yml::QwertConfig::load(&manifest_path)?;

    let hooks = config.hooks_for(&profile, phase);
    for path in hooks {
        let expanded = qwert_yml::expand_tilde(&path);
        println!("[ -f \"{}\" ] && source \"{}\"", expanded, expanded);
    }

    // Convention-based per-profile hook scripts under ~/.qwert/hooks/: <phase>.sh always,
    // <phase>.<profile>.sh only when it matches the active profile. Recursive over tools.
    let hooks_dir = config_dir.join("hooks");
    if hooks_dir.is_dir() {
        let mut found: Vec<std::path::PathBuf> = Vec::new();
        collect_hook_scripts(&hooks_dir, phase, &profile, &mut found);
        for path in found {
            let expanded = path.to_string_lossy().to_string();
            println!("[ -f \"{}\" ] && source \"{}\"", expanded, expanded);
        }
    }

    Ok(())
}