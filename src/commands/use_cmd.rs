use anyhow::Result;

use crate::config::qwert_yml;
use crate::recipe::{index, runner};
use crate::ui::printer;

/// Resolve the profile to declare toward: explicit flag, else active machine profile,
/// else "default".
fn resolve_profile(explicit: Option<&str>) -> String {
    if let Some(p) = explicit {
        return p.to_string();
    }
    crate::config::machine::MachineIdentity::load()
        .map(|m| m.active_profile().to_string())
        .unwrap_or_else(|_| qwert_yml::PROFILE_DEFAULT.to_string())
}

pub fn use_tool(name: &str, version: Option<&str>, profile: Option<&str>, no_install: bool) -> Result<()> {
    let profile = resolve_profile(profile);
    let manifest_path = qwert_yml::manifest_path();
    let mut config = qwert_yml::QwertConfig::load(&manifest_path)?;

    if config.has_tool_in(&profile, name) && version.is_none() {
        printer::info(&format!("{} is already declared in qwert.yml ({})", name, profile));
    } else {
        config.add_tool(&profile, name, version);
        config.save(&manifest_path)?;
        let ver_label = version.unwrap_or("latest");
        printer::ok(name, &format!("added to qwert.yml ({}/{})", profile, ver_label));
    }

    // Seed the config source from the recipe's default `from`, if the tool has a
    // recipe setup and no source is declared yet. Runs before any --no-install
    // early return so the config is explicit and editable regardless of install.
    let recipes_dir = index::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;
    crate::commands::recipes_cmd::update_silent();
    crate::plugins::ensure_clones().ok();

    if config.config_source_for(&profile, name).is_none() {
        let source_from_recipe = index::find(name, &recipes_dir)
            .and_then(|r| r.setup)
            .and_then(|s| s.from);
        if let Some(from) = source_from_recipe {
            config.set_config_source(&profile, name, &from);
            config.save(&manifest_path)?;
            printer::info(&format!(
                "config source set to '{}' (recipe default — edit config.yml to override)",
                from
            ));
        }
    }

    if no_install {
        return Ok(());
    }

    let source = config
        .config_source_for(&profile, name)
        .map(|s| std::path::PathBuf::from(qwert_yml::expand_tilde(s)));

    match index::find(name, &recipes_dir) {
        Some(recipe) => {
            runner::install_with_output(&recipe, &recipes_dir);
            if recipe.setup.is_some() {
                runner::setup_with_output(&recipe, source.as_deref());
            } else if let Some(inline) = config.inline_setup_of(name) {
                runner::setup_inline_with_output(name, inline, source.as_deref());
            }
        }
        None => {
            // No recipe — install via platform default adapter
            if crate::platform::which(name) {
                printer::ok(name, "already installed");
            } else {
                match crate::adapters::default_adapter() {
                    Some(adapter) => {
                        if let Err(e) = crate::platform::run_cmd(&adapter.install_cmd(name)) {
                            printer::failed(name, &e.to_string());
                        } else {
                            printer::ok(name, "installed");
                        }
                    }
                    None => printer::failed(name, "no package manager available on this platform"),
                }
            }
            // Run inline setup if defined
            if let Some(inline) = config.inline_setup_of(name) {
                runner::setup_inline_with_output(name, inline, source.as_deref());
            }
        }
    }

    if let Err(e) = crate::platform::ensure_shell() {
        printer::warning(&format!("could not update shell rc: {}", e));
    }

    Ok(())
}

pub fn use_script(hook: &str, path: &str, profile: Option<&str>) -> Result<()> {
    let profile = resolve_profile(profile);
    let manifest_path = qwert_yml::manifest_path();
    let mut config = qwert_yml::QwertConfig::load(&manifest_path)?;

    config.add_hook(&profile, hook, path);
    config.save(&manifest_path)?;

    printer::ok("script", &format!("added to {} hook ({}) in qwert.yml", hook, profile));
    printer::info("Restart your shell or run `source ~/.zshrc` to apply.");

    Ok(())
}