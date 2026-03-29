use super::super::AptAdapter;
use super::super::PackageAdapter;

#[test]
fn name_returns_apt() {
    let adapter = AptAdapter;
    assert_eq!(adapter.name(), "apt");
}

#[test]
fn install_cmd_returns_apt_install_pkg() {
    let adapter = AptAdapter;
    assert_eq!(adapter.install_cmd("tmux"), "sudo apt-get install -y tmux");
}

#[test]
fn upgrade_cmd_returns_apt_upgrade_pkg() {
    let adapter = AptAdapter;
    assert_eq!(adapter.upgrade_cmd("tmux"), "sudo apt-get install -y tmux");
}

#[test]
fn uninstall_cmd_returns_apt_remove_pkg() {
    let adapter = AptAdapter;
    assert_eq!(adapter.uninstall_cmd("tmux"), "sudo apt-get remove -y tmux");
}

#[test]
fn available_returns_bool() {
    let _ = AptAdapter.available();
}
