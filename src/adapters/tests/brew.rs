use super::super::BrewAdapter;
use super::super::PackageAdapter;

#[test]
fn name_returns_brew() {
    let adapter = BrewAdapter;
    assert_eq!(adapter.name(), "brew");
}

#[test]
fn install_cmd_returns_brew_install_pkg() {
    let adapter = BrewAdapter;
    assert_eq!(adapter.install_cmd("tmux"), "brew install tmux");
}

#[test]
fn upgrade_cmd_returns_brew_upgrade_pkg() {
    let adapter = BrewAdapter;
    assert_eq!(adapter.upgrade_cmd("tmux"), "brew upgrade tmux");
}

#[test]
fn uninstall_cmd_returns_brew_uninstall_pkg() {
    let adapter = BrewAdapter;
    assert_eq!(adapter.uninstall_cmd("tmux"), "brew uninstall tmux");
}

#[test]
fn install_cmd_uses_provided_pkg_name() {
    // delta recipe has meta.name = "delta" but brew package = "git-delta"
    let adapter = BrewAdapter;
    assert_eq!(adapter.install_cmd("git-delta"), "brew install git-delta");
}

#[test]
fn available_returns_bool() {
    // smoke test — just ensure it doesn't panic
    let _ = BrewAdapter.available();
}
