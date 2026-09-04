use super::materialize;
use std::fs;
use std::path::{Path, PathBuf};

fn write_file(dir: &Path, rel: &str, content: &str) {
    let p = dir.join(rel);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(p, content).unwrap();
}

#[test]
fn materialize_returns_none_when_no_overrides() {
    // arrange
    let config_dir = build_config("no_overrides");
    write_file(&config_dir.join("nvim"), "init.lua", "base");
    let data_dir = build_data("no_overrides");
    // act
    let result = materialize("nvim", &["dev".into()], &config_dir, &data_dir).unwrap();
    // assert
    assert!(result.is_none());
    fs::remove_dir_all(&config_dir).ok();
    fs::remove_dir_all(&data_dir).ok();
}

#[test]
fn materialize_returns_none_when_base_missing() {
    // arrange
    let config_dir = build_config("missing");
    let data_dir = build_data("missing");
    // act
    let result = materialize("nvim", &["dev".into()], &config_dir, &data_dir).unwrap();
    // assert
    assert!(result.is_none());
    fs::remove_dir_all(&config_dir).ok();
    fs::remove_dir_all(&data_dir).ok();
}

#[test]
fn materialize_merges_base_with_role_override() {
    // arrange
    let config_dir = build_config("merge");
    let nvim = config_dir.join("nvim");
    write_file(&nvim, "init.lua", "base init");
    write_file(&nvim, "lua/plugins.lua", "base plugins");
    write_file(&nvim, "overrides/dev/init.lua", "dev init");
    let data_dir = build_data("merge");
    // act
    let merged = materialize("nvim", &["dev".into()], &config_dir, &data_dir)
        .unwrap()
        .unwrap();
    // assert — base plugin kept, dev init wins
    assert_eq!(
        fs::read_to_string(merged.join("init.lua")).unwrap(),
        "dev init"
    );
    assert_eq!(
        fs::read_to_string(merged.join("lua/plugins.lua")).unwrap(),
        "base plugins"
    );
    assert!(!merged.join("overrides").exists());
    fs::remove_dir_all(&config_dir).ok();
    fs::remove_dir_all(&data_dir).ok();
}

#[test]
fn materialize_last_role_wins() {
    // arrange
    let config_dir = build_config("last");
    let nvim = config_dir.join("nvim");
    write_file(&nvim, "init.lua", "base");
    write_file(&nvim, "overrides/dev/init.lua", "dev version");
    write_file(&nvim, "overrides/server/init.lua", "server version");
    let data_dir = build_data("last");
    let roles = vec!["dev".into(), "server".into()];
    // act — server applied last
    let merged = materialize("nvim", &roles, &config_dir, &data_dir)
        .unwrap()
        .unwrap();
    // assert
    assert_eq!(
        fs::read_to_string(merged.join("init.lua")).unwrap(),
        "server version"
    );
    fs::remove_dir_all(&config_dir).ok();
    fs::remove_dir_all(&data_dir).ok();
}

#[test]
fn materialize_recreates_dir_every_time() {
    // arrange — first run with a role, then fresh roles with no overrides for one of them
    let config_dir = build_config("recreate");
    let nvim = config_dir.join("nvim");
    write_file(&nvim, "init.lua", "base");
    write_file(&nvim, "overrides/dev/init.lua", "dev init");
    write_file(&nvim, "overrides/server/extra.txt", "server extra");
    let data_dir = build_data("recreate");
    // act — first materialize with [dev, server]
    let m1 = materialize("nvim", &["dev".into(), "server".into()], &config_dir, &data_dir)
        .unwrap()
        .unwrap();
    assert!(m1.join("extra.txt").exists());
    // second run only with [dev] — recreates from scratch, dropping server's extra.txt
    let m2 = materialize("nvim", &["dev".into()], &config_dir, &data_dir)
        .unwrap()
        .unwrap();
    // assert
    assert!(!m2.join("extra.txt").exists());
    assert!(m2.join("init.lua").exists());
    fs::remove_dir_all(&config_dir).ok();
    fs::remove_dir_all(&data_dir).ok();
}

fn build_config(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!("qwert_merge_config_{}_{}", tag, std::process::id()))
}

fn build_data(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!("qwert_merge_data_{}_{}", tag, std::process::id()))
}