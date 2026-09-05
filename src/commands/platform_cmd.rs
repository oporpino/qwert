use anyhow::Result;

use crate::ui::printer;

/// Map a qwert-style platform name (macos|debian|arch) to a yuiop PM name.
fn to_pm(platform: &str) -> Option<&'static str> {
    match platform.trim().to_ascii_lowercase().as_str() {
        "macos" | "mac" | "darwin" | "brew" => Some("brew"),
        "debian" | "ubuntu" | "debian-based" | "apt" => Some("apt"),
        "arch" | "manjaro" | "arch-based" | "pacman" => Some("pacman"),
        _ => None,
    }
}

/// Show the current platform (no args) or set it explicitly.
///
/// Platform detection and persistence are owned by `yuiop` — qwert only reports
/// the effective platform and forwards the override:
///   `qwert platform arch` → `yuiop platform pacman`
pub fn run(platform: Option<&str>) -> Result<()> {
    let Some(platform) = platform else {
        match crate::adapters::yuiop::platform_name() {
            Some(pm) => printer::ok("platform", &format!("{} (managed by yuiop)", pm)),
            None => {
                printer::failed("platform", "could not detect the platform");
                crate::adapters::yuiop::available().then(|| {
                    printer::info("Run `yuiop platform <brew|apt|pacman>` to set it explicitly.");
                });
            }
        }
        printer::info("Set the platform explicitly with `qwert platform <macos|debian|arch>`.");
        return Ok(());
    };

    let pm = to_pm(platform).ok_or_else(|| {
        anyhow::anyhow!("unknown platform '{}' — use one of: macos, debian, arch", platform)
    })?;

    crate::adapters::yuiop::set_platform(pm).map_err(anyhow::Error::msg)?;
    printer::ok("platform", &format!("{} saved (yuiop)", pm));
    printer::info("Run `qwert apply` to apply this machine's setup.");
    Ok(())
}