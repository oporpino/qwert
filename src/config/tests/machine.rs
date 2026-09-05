use crate::config::machine::MachineIdentity;
use std::fs;

#[test]
fn from_env_parses_profile() {
    // arrange
    let env = "dev";
    // act
    let identity = MachineIdentity::from_env(Some(env), None);
    // assert
    assert_eq!(identity.profile.as_deref(), Some("dev"));
}

#[test]
fn from_env_empty_is_none() {
    // arrange
    let env = "  ";
    // act
    let identity = MachineIdentity::from_env(Some(env), None);
    // assert
    assert_eq!(identity.profile, None);
}

#[test]
fn from_env_parses_platform() {
    // arrange
    let platform = "arch";
    // act
    let identity = MachineIdentity::from_env(None, Some(platform));
    // assert
    assert_eq!(identity.platform.as_deref(), Some("arch"));
}

#[test]
fn load_returns_default_when_file_missing() {
    // arrange
    let path = std::env::temp_dir().join("qwert_machine_nonexistent.yml");
    // act
    let identity = MachineIdentity::load_from(&path).unwrap();
    // assert
    assert_eq!(identity.profile, None);
}

#[test]
fn save_to_and_load_from_roundtrip() {
    // arrange
    let path = std::env::temp_dir().join("qwert_machine_roundtrip.yml");
    let identity = MachineIdentity { profile: Some("dev".into()), platform: None };
    // act
    identity.save_to(&path).unwrap();
    let loaded = MachineIdentity::load_from(&path).unwrap();
    fs::remove_file(&path).ok();
    // assert
    assert_eq!(loaded.profile.as_deref(), Some("dev"));
}

#[test]
fn env_override_wins_over_file() {
    // arrange — QWERT_PROFILE takes precedence over machine.yml contents
    std::env::set_var("QWERT_PROFILE", "dev");
    // act
    let loaded = MachineIdentity::load().unwrap();
    std::env::remove_var("QWERT_PROFILE");
    // assert
    assert_eq!(loaded.profile.as_deref(), Some("dev"));
}

#[test]
fn set_profile_replaces_existing() {
    // arrange
    let mut identity = MachineIdentity { profile: Some("dev".into()), platform: None };
    // act
    identity.set_profile("server".into());
    // assert
    assert_eq!(identity.profile.as_deref(), Some("server"));
}

#[test]
fn active_profile_falls_back_to_default() {
    // arrange
    let identity = MachineIdentity { profile: None, platform: None };
    // act
    let active = identity.active_profile();
    // assert
    assert_eq!(active, crate::config::qwert_yml::PROFILE_DEFAULT);
}

#[test]
fn active_profile_returns_set_profile() {
    // arrange
    let identity = MachineIdentity { profile: Some("server".into()), platform: None };
    // act
    let active = identity.active_profile();
    // assert
    assert_eq!(active, "server");
}