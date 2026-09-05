use anyhow::Result;

use crate::config::machine;
use crate::ui::printer;

/// Show the current platform (no args) or set it explicitly.
/// `qwert platform arch` pins this machine to Arch so package recipes always resolve
/// through pacman, even when auto-detection misses. Saved to machine.yml in
/// ~/.local/share/qwert/.
pub fn run(platform: Option<&str>) -> Result<()> {
    let mut machine_identity = machine::MachineIdentity::load()?;

    let Some(platform) = platform else {
        let detected = crate::platform::detect();
        match machine_identity.platform.as_deref() {
            Some(p) => printer::ok("platform", &format!("{} (override)  [detected: {}]", p, detected)),
            None => printer::ok("platform", &format!("{} (auto)", detected)),
        }
        printer::info("Set the platform explicitly with `qwert platform <macos|debian|arch>`.");
        return Ok(());
    };

    let normalized = match platform.trim().to_ascii_lowercase().as_str() {
        "macos" | "mac" | "darwin" => "macos",
        "debian" | "ubuntu" | "debian-based" | "apt" => "debian",
        "arch" | "manjaro" | "arch-based" | "pacman" => "arch",
        other => {
            anyhow::bail!(
                "unknown platform '{}' — use one of: macos, debian, arch",
                other
            );
        }
    };

    machine_identity.set_platform(normalized.to_string());
    machine_identity.save()?;
    printer::ok("platform", &format!("{} saved to {}", normalized, machine::machine_path().display()));
    printer::info("Run `qwert apply` to apply this machine's setup.");
    Ok(())
}