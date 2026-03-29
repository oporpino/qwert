use super::super::BrewAdapter;
use super::super::PackageAdapter;

#[test]
fn install_cmd_returns_brew_install_pkg() {
    // arrange
    let adapter = BrewAdapter;
    // act
    let cmd = adapter.install_cmd("tmux");
    // assert
    assert_eq!(cmd, "brew install tmux");
}

#[test]
fn upgrade_cmd_returns_brew_upgrade_pkg() {
    // arrange
    let adapter = BrewAdapter;
    // act
    let cmd = adapter.upgrade_cmd("tmux");
    // assert
    assert_eq!(cmd, "brew upgrade tmux");
}

#[test]
fn uninstall_cmd_returns_brew_uninstall_pkg() {
    // arrange
    let adapter = BrewAdapter;
    // act
    let cmd = adapter.uninstall_cmd("tmux");
    // assert
    assert_eq!(cmd, "brew uninstall tmux");
}

#[test]
fn install_cmd_uses_provided_pkg_name() {
    // arrange — delta recipe: meta.name = "delta" but brew package = "git-delta"
    let adapter = BrewAdapter;
    // act
    let cmd = adapter.install_cmd("git-delta");
    // assert
    assert_eq!(cmd, "brew install git-delta");
}

#[test]
fn available_returns_bool() {
    // arrange
    let adapter = BrewAdapter;
    // act / assert — smoke test
    let _ = adapter.available();
}
