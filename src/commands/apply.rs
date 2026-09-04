use anyhow::Result;

use crate::config::{machine, qwert_yml, state_yml};
use crate::recipe::{index, runner};
use crate::ui::printer;

pub fn run(tool: Option<&str>, dry_run: bool) -> Result<()> {
    let manifest_path = qwert_yml::manifest_path();
    let state_path = state_yml::state_path();

    let config = qwert_yml::QwertConfig::load(&manifest_path)?;
    let mut state = state_yml::QwertState::load(&state_path)?;
    let machine_identity = machine::MachineIdentity::load()?;
    let profile = machine_identity.active_profile().to_string();

    let recipes_dir = index::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;

    printer::h1("Applying machine setup...");
    if !config.profiles.contains_key(&profile) {
        printer::warning(&format!(
            "profile '{}' is not declared in qwert.yml (available: {})",
            profile,
            config.profile_names().join(", ")
        ));
    }
    printer::info(&format!("profile: {}", profile));
    printer::blank();

    // Tools active for this machine's profile.
    let active_names = config.tool_names_for_profile(&profile);

    let mut done = 0;
    let mut failed = 0;
    let mut orphan_done = 0;

    // ---- Phase 1: uninstall orphans (installed but no longer active) ----
    if tool.is_none() {
        let orphans: Vec<String> = state
            .orphans(&active_names)
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        if !orphans.is_empty() {
            printer::h2("Uninstalling orphans");
            printer::blank();
        }

        for name in &orphans {
            if dry_run {
                printer::bullet(&format!("would uninstall: {}", name));
                continue;
            }
            // The platform's own package manager (brew/apt/pacman) is never a tool —
            // it may appear in state from a bootstrap step. Keep it, just untrack.
            if crate::adapters::is_package_manager(name) {
                state.mark_removed(name);
                printer::ok(name, "kept (package manager)");
                orphan_done += 1;
                continue;
            }
            match index::find(name, &recipes_dir) {
                Some(recipe) => {
                    if runner::uninstall_with_output(&recipe) {
                        state.mark_removed(name);
                        orphan_done += 1;
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
                                orphan_done += 1;
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
        if !orphans.is_empty() {
            printer::blank();
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

    // ---- Phase 2: install all tools ----
    if dry_run {
        printer::h2("Would install");
        printer::blank();
        for name in &tools {
            printer::bullet(&format!("{}", name));
        }
        printer::blank();
        printer::h2("Would setup");
        printer::blank();
        for name in &tools {
            printer::bullet(&format!("{}", name));
        }
        printer::blank();
        return Ok(());
    }

    printer::h2("Installing tools");
    printer::blank();

    for name in &tools {
        match index::find(name, &recipes_dir) {
            Some(recipe) => {
                if recipe.setup_only {
                    // Setup-only recipes (e.g. local agents/skills) skip install.
                    printer::ok(name, "setup-only (no package)");
                    done += 1;
                    continue;
                }
                let installed = runner::install_with_output(&recipe, &recipes_dir);
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
            }
        }
    }

    printer::blank();

    // ---- Phase 3: setup all tools ----
    printer::h2("Setting up tools");
    printer::blank();

    for name in &tools {
        let source = config
            .config_source_for(&profile, name)
            .map(|s| std::path::PathBuf::from(qwert_yml::expand_tilde(s)));
        match index::find(name, &recipes_dir) {
            Some(recipe) => {
                if recipe.setup.is_some() {
                    runner::setup_with_output(&recipe, source.as_deref());
                } else if let Some(inline) = config.inline_setup_of(name) {
                    runner::setup_inline_with_output(name, inline, source.as_deref());
                }
            }
            None => {
                if let Some(inline) = config.inline_setup_of(name) {
                    runner::setup_inline_with_output(name, inline, source.as_deref());
                }
            }
        }
    }

    printer::blank();

    state.save(&state_path)?;
    if orphan_done > 0 {
        printer::info(&format!("{} orphan(s) uninstalled", orphan_done));
    }
    printer::summary(done, tools.len(), failed);

    Ok(())
}