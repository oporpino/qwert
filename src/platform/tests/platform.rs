use super::*;

#[test]
fn platform_for_pm_maps_brew_to_macos() {
    // arrange
    let pm = Some("brew");
    // act
    let platform = platform_for_pm(pm);
    // assert
    assert_eq!(platform, Platform::MacOS);
}

#[test]
fn platform_for_pm_maps_apt_to_debian() {
    // arrange
    let pm = Some("apt");
    // act
    let platform = platform_for_pm(pm);
    // assert
    assert_eq!(platform, Platform::Debian);
}

#[test]
fn platform_for_pm_maps_pacman_to_arch() {
    // arrange
    let pm = Some("pacman");
    // act
    let platform = platform_for_pm(pm);
    // assert
    assert_eq!(platform, Platform::Arch);
}

#[test]
fn platform_for_pm_unknown_name_is_unknown_platform() {
    // arrange
    let pm = Some("whatever");
    // act
    let platform = platform_for_pm(pm);
    // assert
    assert_eq!(platform, Platform::Unknown);
}

#[test]
fn platform_for_pm_none_is_unknown_platform() {
    // arrange
    let pm = None;
    // act
    let platform = platform_for_pm(pm);
    // assert
    assert_eq!(platform, Platform::Unknown);
}

#[test]
fn installer_uses_system_layout() {
    // arrange — the qwert install layout is `/opt/qwert/bin` on every OS
    // act
    let inst = installer();
    // assert
    assert_eq!(inst.binary_path().to_string_lossy(), "/opt/qwert/bin/qwert");
    assert_eq!(inst.symlink_path().to_string_lossy(), "/usr/local/bin/qwert");
}