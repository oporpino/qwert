use anyhow::Result;

use crate::config::{machine, qwert_yml};
use crate::ui::printer;

/// Show the current machine profile (no args) or set it.
pub fn run(name: Option<&str>) -> Result<()> {
    let mut machine_identity = machine::MachineIdentity::load()?;

    let Some(name) = name else {
        let profile = machine_identity.active_profile();
        let manifest_path = qwert_yml::manifest_path();
        let config = qwert_yml::QwertConfig::load(&manifest_path)?;
        if config.has_profile(profile) {
            printer::ok("profile", profile);
            printer::info("Run `qwert apply` to apply this machine's profile.");
        } else {
            printer::warning(&format!(
                "profile '{}' is not declared in qwert.yml (available: {})",
                profile,
                config.profile_names().join(", ")
            ));
            printer::info("Set it with `qwert profile <name>`.");
        }
        return Ok(());
    };

    machine_identity.set_profile(name.to_string());
    machine_identity.save()?;

    let manifest_path = qwert_yml::manifest_path();
    let config = qwert_yml::QwertConfig::load(&manifest_path)?;
    if !config.has_profile(name) {
        printer::warning(&format!(
            "profile '{}' is not declared in qwert.yml (available: {})",
            name,
            config.profile_names().join(", ")
        ));
    }

    printer::ok("profile", &name);
    printer::info("Run `qwert apply` to apply this machine's profile.");
    Ok(())
}