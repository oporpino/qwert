use super::PackageAdapter;

pub struct PacmanAdapter;

impl PackageAdapter for PacmanAdapter {
    fn name(&self) -> &str { "pacman" }
    fn available(&self) -> bool { crate::platform::which("pacman") }
    fn install_cmd(&self, pkg: &str) -> String { format!("sudo pacman -S --noconfirm {}", pkg) }
    fn upgrade_cmd(&self, _pkg: &str) -> String { "sudo pacman -Su --noconfirm".to_string() }
    fn uninstall_cmd(&self, pkg: &str) -> String { format!("sudo pacman -R --noconfirm {}", pkg) }
}

#[cfg(test)]
#[path = "tests/pacman.rs"]
mod tests;
