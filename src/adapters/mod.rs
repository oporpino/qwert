pub mod yuiop;

/// Is this name one of the system package managers (never a user tool)?
///
/// Guards orphan cleanup: brew/apt/pacman may appear in state.yml from a
/// bootstrap step, but qwert must never uninstall the very manager it depends on.
pub fn is_package_manager(name: &str) -> bool {
    matches!(name, "brew" | "apt" | "apt-get" | "pacman")
}