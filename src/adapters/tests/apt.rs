use super::super::AptAdapter;
use super::super::PackageAdapter;

#[test]
fn name_returns_apt() {
    // arrange
    let adapter = AptAdapter;
    // act
    let name = adapter.name();
    // assert
    assert_eq!(name, "apt");
}

#[test]
fn install_cmd_returns_apt_install_pkg() {
    // arrange
    let adapter = AptAdapter;
    // act
    let cmd = adapter.install_cmd("tmux");
    // assert
    assert_eq!(cmd, "sudo apt-get install -y tmux");
}

#[test]
fn upgrade_cmd_returns_apt_upgrade_pkg() {
    // arrange
    let adapter = AptAdapter;
    // act
    let cmd = adapter.upgrade_cmd("tmux");
    // assert
    assert_eq!(cmd, "sudo apt-get install -y tmux");
}

#[test]
fn uninstall_cmd_returns_apt_remove_pkg() {
    // arrange
    let adapter = AptAdapter;
    // act
    let cmd = adapter.uninstall_cmd("tmux");
    // assert
    assert_eq!(cmd, "sudo apt-get remove -y tmux");
}

#[test]
fn available_returns_bool() {
    // arrange
    let adapter = AptAdapter;
    // act / assert — smoke test
    let _ = adapter.available();
}
