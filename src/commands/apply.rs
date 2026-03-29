use anyhow::Result;

use crate::config::{qwert_yml, state_yml};
use crate::recipe::{index, runner};
use crate::ui::printer;

pub fn run(tool: Option<&str>, dry_run: bool) -> Result<()> {
    let manifest_path = qwert_yml::manifest_path();
    let state_path = state_yml::state_path();

    let config = qwert_yml::QwertConfig::load(&manifest_path)?;
    let mut state = state_yml::QwertState::load(&state_path)?;

    let recipes_dir = index::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;
    let config_dir = qwert_yml::config_dir();

    printer::h1("Applying machine setup...");
    printer::blank();

    let mut done = 0;
    let mut failed = 0;

    // Uninstall orphans
    if tool.is_none() {
        let orphans: Vec<String> = state.orphans(&config.tool_names())
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        for name in &orphans {
            if dry_run {
                printer::bullet(&format!("would uninstall: {}", name));
                continue;
            }
            match index::find(name, &recipes_dir) {
                Some(recipe) => {
                    if runner::uninstall_with_output(&recipe) {
                        state.mark_removed(name);
                        done += 1;
                    } else {
                        failed += 1;
                    }
                }
                None => {
                    // No recipe — try default adapter
                    match crate::adapters::default_adapter() {
                        Some(adapter) => {
                            if crate::platform::run_cmd(&adapter.uninstall_cmd(name)).is_ok() {
                                state.mark_removed(name);
                                printer::ok(name, "uninstalled");
                                done += 1;
                            } else {
                                printer::failed(name, "uninstall failed — remove manually");
                                failed += 1;
                            }
                        }
                        None => {
                            printer::failed(name, "no recipe and no package manager — remove manually");
                            failed += 1;
                        }
                    }
                }
            }
        }
    }

    // Install + setup tools declared in manifest
    let tool_names = config.tool_names();
    let tools: Vec<&str> = if let Some(t) = tool {
        vec![t]
    } else {
        tool_names.iter().map(|s| s.as_str()).collect()
    };

    if tools.is_empty() && state.orphans(&config.tool_names()).is_empty() {
        printer::info("No tools declared. Run `qwert use <tool>` to add one.");
        return Ok(());
    }

    for name in &tools {
        if dry_run {
            printer::bullet(&format!("would install: {}", name));
            printer::bullet(&format!("would setup: {}", name));
            continue;
        }
        match index::find(name, &recipes_dir) {
            Some(recipe) => {
                let installed = runner::install_with_output(&recipe, &recipes_dir);
                if recipe.setup.is_some() {
                    runner::setup_with_output(&recipe, &config_dir);
                } else if let Some(inline) = config.setup_of(name) {
                    runner::setup_inline_with_output(name, inline, &config_dir);
                }
                if installed {
                    let version = runner::installed_version(&recipe);
                    state.mark_installed(name, version.as_deref());
                    done += 1;
                } else {
                    failed += 1;
                }
            }
            None => {
                if crate::platform::which(name) {
                    let version = crate::platform::version_of(name, "--version");
                    state.mark_installed(name, version.as_deref());
                    printer::ok(name, "already installed");
                    done += 1;
                } else {
                    match crate::adapters::default_adapter() {
                        Some(adapter) => {
                            if crate::platform::run_cmd(&adapter.install_cmd(name)).is_ok() {
                                let version = crate::platform::version_of(name, "--version");
                                state.mark_installed(name, version.as_deref());
                                printer::ok(name, "installed");
                                done += 1;
                            } else {
                                printer::failed(name, "install failed");
                                failed += 1;
                            }
                        }
                        None => {
                            printer::failed(name, "no recipe and no package manager available");
                            failed += 1;
                        }
                    }
                }
                // Run inline setup if defined
                if let Some(inline) = config.setup_of(name) {
                    runner::setup_inline_with_output(name, inline, &config_dir);
                }
            }
        }
    }

    if !dry_run {
        state.save(&state_path)?;
        printer::summary(done, tools.len(), failed);
    }

    Ok(())
}
