use crate::config::machine::MachineIdentity;
use std::fs;

#[test]
fn from_env_parses_comma_separated_roles() {
    // arrange
    let env = "dev, server, macos";
    // act
    let identity = MachineIdentity::from_env(env);
    // assert
    assert_eq!(identity.roles, vec!["dev", "server", "macos"]);
}

#[test]
fn from_env_trims_and_drops_empty_entries() {
    // arrange
    let env = "dev, ,,server,";
    // act
    let identity = MachineIdentity::from_env(env);
    // assert
    assert_eq!(identity.roles, vec!["dev", "server"]);
}

#[test]
fn load_returns_default_when_file_missing() {
    // arrange
    let path = std::env::temp_dir().join("qwert_machine_nonexistent.yml");
    // act
    let identity = MachineIdentity::load_from(&path).unwrap();
    // assert
    assert!(identity.roles.is_empty());
}

#[test]
fn save_to_and_load_from_roundtrip() {
    // arrange
    let path = std::env::temp_dir().join("qwert_machine_roundtrip.yml");
    let identity = MachineIdentity { roles: vec!["dev".into(), "server".into()] };
    // act
    identity.save_to(&path).unwrap();
    let loaded = MachineIdentity::load_from(&path).unwrap();
    fs::remove_file(&path).ok();
    // assert
    assert_eq!(loaded.roles, vec!["dev", "server"]);
}

#[test]
fn env_override_wins_over_file() {
    // arrange — QWERT_ROLES takes precedence over machine.yml contents
    std::env::set_var("QWERT_ROLES", "dev, macos");
    // act
    let loaded = MachineIdentity::load().unwrap();
    std::env::remove_var("QWERT_ROLES");
    // assert
    assert_eq!(loaded.roles, vec!["dev", "macos"]);
}

#[test]
fn set_roles_replaces_existing_roles() {
    // arrange
    let mut identity = MachineIdentity { roles: vec!["dev".into()] };
    // act
    identity.set_roles(vec!["server".into()]);
    // assert
    assert_eq!(identity.roles, vec!["server"]);
}