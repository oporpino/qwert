use anyhow::Result;

use crate::config::{machine, qwert_yml};
use crate::ui::printer;

/// Show current machine roles (no args) or set/validate them.
pub fn run(roles: &[String]) -> Result<()> {
    let mut machine_identity = machine::MachineIdentity::load()?;

    if roles.is_empty() {
        if machine_identity.roles.is_empty() {
            printer::info(
                "No roles configured for this machine. Use `qwert machine <roles>` or set QWERT_ROLES.",
            );
        } else {
            printer::ok("roles", &machine_identity.roles.join(", "));
            printer::info("Run `qwert apply` to apply this machine's roles.");
        }
        return Ok(());
    }

    machine_identity.set_roles(roles.to_vec());
    machine_identity.save()?;

    let manifest_path = qwert_yml::manifest_path();
    let config = qwert_yml::QwertConfig::load(&manifest_path)?;
    let available = config.role_sections();

    for r in roles {
        if !available.contains(r) {
            printer::warning(&format!("role '{}' has no tools declared in qwert.yml", r));
        }
    }

    printer::ok("roles", &roles.join(", "));
    printer::info("Run `qwert apply` to apply this machine's roles.");
    Ok(())
}