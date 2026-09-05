//! yuiop — the internal package-manager resolution layer.
//!
//! `qwert` + `yuiop` = `QWERTYUIOP`. Recipes are package-manager agnostic: they say
//! *which* package (optionally with a name per PM via `packages`); yuiop picks *how*
//! to install it — brew on macOS, apt on Debian, pacman on Arch. This is the single
//! place that maps a platform to its package manager.

use super::{AptAdapter, BrewAdapter, PackageAdapter, PacmanAdapter};
use crate::platform::{self, Platform};
use crate::recipe::schema::{RecipeKind, RecipeMeta};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pm {
    Brew,
    Apt,
    Pacman,
}

impl Pm {
    /// The package manager for the current platform, if any.
    pub fn current() -> Option<Pm> {
        match platform::detect() {
            Platform::MacOS => Some(Pm::Brew),
            Platform::Debian => Some(Pm::Apt),
            Platform::Arch => Some(Pm::Pacman),
            Platform::Unknown => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Pm::Brew => "brew",
            Pm::Apt => "apt",
            Pm::Pacman => "pacman",
        }
    }

    pub fn adapter(self) -> Box<dyn PackageAdapter> {
        match self {
            Pm::Brew => Box::new(BrewAdapter),
            Pm::Apt => Box::new(AptAdapter),
            Pm::Pacman => Box::new(PacmanAdapter),
        }
    }

    /// Is `pkg` installed, per the PM's own database?
    pub fn installed(self, pkg: &str) -> bool {
        let cmd = match self {
            Pm::Brew => format!("brew list {pkg} &>/dev/null"),
            Pm::Apt => format!("dpkg -s {pkg} &>/dev/null"),
            Pm::Pacman => format!("pacman -Q {pkg} &>/dev/null"),
        };
        platform::run_cmd_capture(&cmd).is_ok()
    }
}

/// A package resolved to a concrete PM + package name on the current platform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    pub pm: Pm,
    pub pkg: String,
}

/// The recipe's effective package kind?
pub fn is_package_kind(kind: &RecipeKind) -> bool {
    kind.is_package()
}

/// Human label for a recipe's manager: the resolved platform PM for package
/// recipes, otherwise the recipe kind (custom/qwert/…). Used in status output.
pub fn resolve_label(meta: &RecipeMeta) -> String {
    if is_package_kind(&meta.kind) {
        resolve(meta)
            .map(|r| r.pm.label().to_string())
            .unwrap_or_else(|| meta.kind.to_string())
    } else {
        meta.kind.to_string()
    }
}

/// Resolve a recipe's package for the current platform's PM.
///
/// Resolution order: `packages.<pm>` → legacy `pkg` (brew only) → `name`.
pub fn resolve(meta: &RecipeMeta) -> Option<Resolved> {
    let pm = Pm::current()?;
    let pkg = meta
        .packages
        .as_ref()
        .and_then(|map| map.get(pm.label()).cloned())
        .or_else(|| (pm == Pm::Brew).then(|| meta.pkg.clone()).flatten())
        .unwrap_or_else(|| meta.name.clone());
    Some(Resolved { pm, pkg })
}

/// Is the recipe's package already installed? `None` when the platform has no PM
/// (caller falls back to `which`).
pub fn registered(meta: &RecipeMeta) -> Option<bool> {
    resolve(meta).map(|r| r.pm.installed(&r.pkg))
}

fn run(meta: &RecipeMeta, op: fn(Box<dyn PackageAdapter>, &str) -> String) -> Result<(), String> {
    let r = resolve(meta).ok_or_else(no_pm_hint)?;
    let adapter = r.pm.adapter();
    adapter.ensure().map_err(|e| e.to_string())?;
    let cmd = op(adapter, &r.pkg);
    platform::run_cmd(&cmd).map_err(|e| e.to_string())
}

/// Message used when no package manager can be resolved for the machine.
fn no_pm_hint() -> String {
    "could not detect a package manager for this machine — run `qwert platform <macos|debian|arch>` to set it explicitly"
        .to_string()
}

pub fn install(meta: &RecipeMeta) -> Result<(), String> {
    run(meta, |a, pkg| a.install_cmd(pkg))
}

pub fn upgrade(meta: &RecipeMeta) -> Result<(), String> {
    run(meta, |a, pkg| a.upgrade_cmd(pkg))
}

pub fn uninstall(meta: &RecipeMeta) -> Result<(), String> {
    run(meta, |a, pkg| a.uninstall_cmd(pkg))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(name: &str, kind: RecipeKind) -> RecipeMeta {
        RecipeMeta {
            name: name.to_string(),
            version: "1.0.0".into(),
            description: "test".into(),
            kind,
            depends: vec![],
            packages: None,
            pkg: None,
        }
    }

    #[test]
    fn package_kinds_are_package() {
        // arrange
        let kinds = [RecipeKind::Package, RecipeKind::Brew, RecipeKind::Apt, RecipeKind::Pacman];
        // act + assert
        for k in &kinds {
            assert!(is_package_kind(k), "{} should resolve as package", k);
        }
    }

    #[test]
    fn custom_kinds_are_not_package() {
        // arrange
        let kinds = [RecipeKind::Custom, RecipeKind::Qwert];
        // act + assert
        for k in &kinds {
            assert!(!is_package_kind(k), "{} should be custom", k);
        }
    }

    #[test]
    fn resolve_uses_packages_entry_for_current_pm() {
        // arrange
        let mut m = meta("asdf", RecipeKind::Package);
        m.packages = Some(
            [("brew", "asdf-brew"), ("apt", "asdf-apt"), ("pacman", "asdf-pacman")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        );
        // act
        let r = resolve(&m);
        // assert
        let r = r.expect("current platform should map to a PM");
        let expected = format!("asdf-{}", r.pm.label());
        assert_eq!(r.pkg, expected, "uses the entry for the current PM");
    }

    #[test]
    fn resolve_falls_back_to_name_when_no_packages_table() {
        // arrange
        let m = meta("fzf", RecipeKind::Package);
        // act
        let r = resolve(&m);
        // assert
        assert_eq!(r.map(|r| r.pkg), Some("fzf".to_string()));
    }

    #[test]
    fn resolve_uses_legacy_pkg_only_for_brew_generic_name_otherwise() {
        // arrange
        let mut m = meta("opencode", RecipeKind::Brew);
        m.pkg = Some("anomalyco/tap/opencode".into());
        // act
        let r = resolve(&m);
        // assert — brew gets the tap, other PMs get the plain name
        let r = r.expect("current platform should map to a PM");
        if r.pm == Pm::Brew {
            assert_eq!(r.pkg, "anomalyco/tap/opencode");
        } else {
            assert_eq!(r.pkg, "opencode");
        }
    }
}