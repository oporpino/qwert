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
}

pub fn for_kind(kind: &RecipeKind) -> Option<Box<dyn PackageAdapter>> {
    match kind {
        RecipeKind::Brew => Some(Box::new(BrewAdapter)),
        RecipeKind::Apt => Some(Box::new(AptAdapter)),
        RecipeKind::Pacman => Some(Box::new(PacmanAdapter)),
        RecipeKind::Qwert => None,
    }
}

#[cfg(test)]
#[path = "tests/mod_tests.rs"]
mod tests;
