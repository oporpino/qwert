use super::PackageAdapter;

pub struct AptAdapter;

impl PackageAdapter for AptAdapter {
    fn available(&self) -> bool { crate::platform::which("apt-get") }
    fn install_cmd(&self, pkg: &str) -> String { format!("sudo apt-get install -y {}", pkg) }
    fn upgrade_cmd(&self, pkg: &str) -> String { format!("sudo apt-get install --only-upgrade -y {}", pkg) }
    fn uninstall_cmd(&self, pkg: &str) -> String { format!("sudo apt-get remove -y {}", pkg) }
}

#[cfg(test)]
#[path = "tests/apt.rs"]
mod tests;
