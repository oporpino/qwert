use super::*;

fn with_platform_env(raw: &str, f: impl FnOnce()) {
    std::env::set_var("QWERT_PLATFORM", raw);
    f();
    std::env::remove_var("QWERT_PLATFORM");
}

#[test]
fn overridden_platform_parses_arch() {
    // arrange
    let mut result = None;
    // act
    with_platform_env("arch", || result = overridden_platform());
    // assert
    assert_eq!(result, Some(Platform::Arch));
}

#[test]
fn overridden_platform_parses_debian_aliases() {
    // arrange
    let mut result = None;
    // act
    with_platform_env("ubuntu", || result = overridden_platform());
    // assert
    assert_eq!(result, Some(Platform::Debian));
}

#[test]
fn overridden_platform_ignores_unknown() {
    // arrange
    let mut result = Some(Platform::MacOS);
    // act
    with_platform_env("windows", || result = overridden_platform());
    // assert
    assert_eq!(result, None);
}

#[test]
fn overridden_platform_none_without_env() {
    // arrange
    std::env::remove_var("QWERT_PLATFORM");
    // act
    let result = overridden_platform();
    // assert
    assert_eq!(result, None);
}