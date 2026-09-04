use anyhow::Result;

use crate::config::{machine, merge, qwert_yml, state_yml};
use crate::recipe::{index, runner};
use crate::ui::printer;

/// Materialize role overrides for a tool → merged `from` path (if any overrides exist).
fn merged_from(tool: &str, roles: &[String], config_dir: &std::path::Path) -> Option<String> {
    let data_dir = crate::platform::data_dir();
    merge::materialize(tool, roles, config_dir, &data_dir)
        .ok()
        .flatten()
        .map(|p| p.to_string_lossy().to_string())
}

pub fn run(tool: Option<&str>, dry_run: bool) -> Result<()> {
    let manifest_path = qwert_yml::manifest_path();
    let state_path = state_yml::state_path();

    let config = qwert_yml::QwertConfig::load(&manifest_path)?;
    let mut state = state_yml::QwertState::load(&state_path)?;
    let roles = machine::MachineIdentity::load()?.roles;

    let recipes_dir = index::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;
    let config_dir = qwert_yml::config_dir();

    printer::h1("Applying machine setup...");
    if !roles.is_empty() {
        printer::info(&format!("roles: {}", roles.join(", ")));
    }
    printer::blank();

    // Tools active for this machine (union of shared + active roles).
    let active_names = config.tool_names_for_roles(&roles);

    let mut done = 0;
    let mut failed = 0;

    // Uninstall orphans: installed but no longer active for this machine.
    if tool.is_none() {
        let orphans: Vec<String> = state
            .orphans(&active_names)
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

    let tools: Vec<&str> = if let Some(t) = tool {
        vec![t]
    } else {
        active_names.iter().map(|s| s.as_str()).collect()
    };

    if tools.is_empty() && state.orphans(&active_names).is_empty() {
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
            Some(mut recipe) => {
                let installed = runner::install_with_output(&recipe, &recipes_dir);
                // Role overrides: point the symlink/copy `from` at the merged view.
                if let Some(from) = merged_from(name, &roles, &config_dir) {
                    if let Some(setup) = recipe.setup.as_mut() {
                        if setup.from.is_none() {
                            setup.from = Some(from);
                        }
                    }
                }
                if recipe.setup.is_some() {
                    runner::setup_with_output(&recipe, &config_dir);
                } else if let Some(inline) = config.setup_of_for_roles(name, &roles) {
                    let mut inline = inline.clone();
                    if inline.from.is_none() {
                        inline.from = merged_from(name, &roles, &config_dir);
                    }
                    runner::setup_inline_with_output(name, &inline, &config_dir);
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
                if let Some(inline) = config.setup_of_for_roles(name, &roles) {
                    let mut inline = inline.clone();
                    if inline.from.is_none() {
                        inline.from = merged_from(name, &roles, &config_dir);
                    }
                    runner::setup_inline_with_output(name, &inline, &config_dir);
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