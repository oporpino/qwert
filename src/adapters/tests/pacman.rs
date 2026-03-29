use super::super::PacmanAdapter;
use super::super::PackageAdapter;

#[test]
fn name_returns_pacman() {
    // arrange
    let adapter = PacmanAdapter;
    // act
    let name = adapter.name();
    // assert
    assert_eq!(name, "pacman");
}

#[test]
fn install_cmd_returns_pacman_install_pkg() {
    // arrange
    let adapter = PacmanAdapter;
    // act
    let cmd = adapter.install_cmd("tmux");
    // assert
    assert_eq!(cmd, "sudo pacman -S --noconfirm tmux");
}

#[test]
fn upgrade_cmd_returns_pacman_upgrade_pkg() {
    // arrange
    let adapter = PacmanAdapter;
    // act
    let cmd = adapter.upgrade_cmd("tmux");
    // assert
    assert_eq!(cmd, "sudo pacman -Su --noconfirm");
}

#[test]
fn uninstall_cmd_returns_pacman_remove_pkg() {
    // arrange
    let adapter = PacmanAdapter;
    // act
    let cmd = adapter.uninstall_cmd("tmux");
    // assert
    assert_eq!(cmd, "sudo pacman -R --noconfirm tmux");
}

#[test]
fn available_returns_bool() {
    // arrange
    let adapter = PacmanAdapter;
    // act / assert — smoke test
    let _ = adapter.available();
}
