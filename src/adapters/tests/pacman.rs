use super::super::PacmanAdapter;
use super::super::PackageAdapter;

#[test]
fn name_returns_pacman() {
    let adapter = PacmanAdapter;
    assert_eq!(adapter.name(), "pacman");
}

#[test]
fn install_cmd_returns_pacman_install_pkg() {
    let adapter = PacmanAdapter;
    assert_eq!(adapter.install_cmd("tmux"), "sudo pacman -S --noconfirm tmux");
}

#[test]
fn upgrade_cmd_returns_pacman_upgrade_pkg() {
    let adapter = PacmanAdapter;
    assert_eq!(adapter.upgrade_cmd("tmux"), "sudo pacman -Su --noconfirm");
}

#[test]
fn uninstall_cmd_returns_pacman_remove_pkg() {
    let adapter = PacmanAdapter;
    assert_eq!(adapter.uninstall_cmd("tmux"), "sudo pacman -R --noconfirm tmux");
}

#[test]
fn available_returns_bool() {
    let _ = PacmanAdapter.available();
}
