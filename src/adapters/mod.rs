pub mod apt;
pub mod brew;
pub mod pacman;

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

pub fn for_kind(kind: &RecipeKind) -> Option<Box<dyn PackageAdapter>> {
    match kind {
        RecipeKind::Brew => Some(Box::new(BrewAdapter)),
        RecipeKind::Apt => Some(Box::new(AptAdapter)),
        RecipeKind::Pacman => Some(Box::new(PacmanAdapter)),
        RecipeKind::Qwert => None,
    }
}

/// Returns the default adapter for the current platform (brew on macOS, apt on Debian).
pub fn default_adapter() -> Option<Box<dyn PackageAdapter>> {
    match crate::platform::detect() {
        crate::platform::Platform::MacOS => Some(Box::new(BrewAdapter)),
        crate::platform::Platform::Debian => Some(Box::new(AptAdapter)),
        crate::platform::Platform::Arch => Some(Box::new(PacmanAdapter)),
        crate::platform::Platform::Unknown => None,
    }
}

#[cfg(test)]
#[path = "tests/mod_tests.rs"]
mod tests;
