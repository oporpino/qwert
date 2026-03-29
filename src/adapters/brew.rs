use super::PackageAdapter;

pub struct BrewAdapter;

impl PackageAdapter for BrewAdapter {
    fn available(&self) -> bool { crate::platform::which("brew") }
    fn install_cmd(&self, pkg: &str) -> String { format!("brew install {}", pkg) }
    fn upgrade_cmd(&self, pkg: &str) -> String { format!("brew upgrade {}", pkg) }
    fn uninstall_cmd(&self, pkg: &str) -> String { format!("brew uninstall {}", pkg) }
}

#[cfg(test)]
#[path = "tests/brew.rs"]
mod tests;
