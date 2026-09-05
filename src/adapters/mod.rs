pub mod apt;
pub mod brew;
pub mod pacman;
pub mod yuiop;

pub use apt::AptAdapter;
pub use brew::BrewAdapter;
pub use pacman::PacmanAdapter;

use crate::recipe::schema::RecipeKind;

pub trait PackageAdapter {
    fn available(&self) -> bool;
    fn install_cmd(&self, pkg: &str) -> String;
    fn upgrade_cmd(&self, pkg: &str) -> String;
    fn uninstall_cmd(&self, pkg: &str) -> String;

    /// Ensure the package manager itself is installed. Returns Ok if available after the call.
    fn ensure(&self) -> anyhow::Result<()> {
        if self.available() {
            return Ok(());
        }
        anyhow::bail!("package manager not available");
    }
}

#[allow(dead_code)] // kept as a convenience for callers that target a specific PM by recipe kind
pub fn for_kind(kind: &RecipeKind) -> Option<Box<dyn PackageAdapter>> {
    match kind {
        RecipeKind::Brew => Some(Box::new(BrewAdapter)),
        RecipeKind::Apt => Some(Box::new(AptAdapter)),
        RecipeKind::Pacman => Some(Box::new(PacmanAdapter)),
        RecipeKind::Package | RecipeKind::Custom | RecipeKind::Qwert => None,
    }
}

/// Is this name one of the package managers qwert knows (never a user tool)?
pub fn is_package_manager(name: &str) -> bool {
    matches!(name, "brew" | "apt" | "apt-get" | "pacman")
}

/// Returns the default adapter for the current platform (brew on macOS, apt on Debian).
/// Single source is `yuiop::Pm::current()` — this delegates to it.
pub fn default_adapter() -> Option<Box<dyn PackageAdapter>> {
    yuiop::Pm::current().map(|pm| pm.adapter())
}

#[cfg(test)]
#[path = "tests/mod_tests.rs"]
mod tests;
