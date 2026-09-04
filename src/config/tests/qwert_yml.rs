use crate::config::qwert_yml::{InlineSetup, QwertConfig, PROFILE_DEFAULT};
use serde_yml::Value;
use std::fs;

// --- Loading: legacy flat config → default profile ---

#[test]
fn flat_tools_load_into_default_profile() {
    // arrange
    let yaml = "tools:\n  tmux: latest\n  git: latest\n";
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // assert
    assert!(config.has_tool_in(PROFILE_DEFAULT, "tmux"));
    assert!(config.declared_anywhere("git"));
    assert_eq!(config.profile_names(), vec![PROFILE_DEFAULT]);
}

#[test]
fn flat_full_entry_keeps_profiles_tools_list() {
    // arrange — legacy full object with version + setup
    let yaml = "tools:\n  neovim:\n    version: \"0.10\"\n";
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // assert
    assert_eq!(config.version_of(PROFILE_DEFAULT, "neovim"), "0.10");
}

#[test]
fn flat_hooks_load_into_default_profile() {
    // arrange
    let yaml = "hooks:\n  init:\n    - ~/env.sh\n";
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // assert
    assert_eq!(config.hooks_for_profile(PROFILE_DEFAULT, "init"), vec!["~/env.sh"]);
}

// --- Loading: catalog + profiles ---

#[test]
fn profiles_load_tools_and_configs() {
    // arrange
    let yaml = "tools:\n  iterm2: latest\n  tmux: v1.0.3\nprofiles:\n  dev:\n    tools: [iterm2, tmux]\n    configs:\n      nvim: ~/.qwert/config/nvim.lua\n  server:\n    tools: [tmux]\n";
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // assert
    assert!(config.has_tool_in("dev", "iterm2"));
    assert!(config.has_tool_in("dev", "tmux"));
    assert!(config.has_tool_in("server", "tmux"));
    assert!(!config.has_tool_in("server", "iterm2"));
    assert_eq!(config.version_of("dev", "tmux"), "v1.0.3");
    assert_eq!(
        config.config_source_for("dev", "nvim"),
        Some("~/.qwert/config/nvim.lua")
    );
    assert_eq!(config.config_source_for("server", "nvim"), None);
}

#[test]
fn profiles_parse_hooks() {
    // arrange
    let yaml = "profiles:\n  dev:\n    hooks:\n      prepare: [~/a.sh]\n      init: [~/b.sh]\n";
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // assert
    assert_eq!(config.hooks_for_profile("dev", "prepare"), vec!["~/a.sh"]);
    assert_eq!(config.hooks_for_profile("dev", "init"), vec!["~/b.sh"]);
}

#[test]
fn profiles_accept_profiles_only() {
    // arrange — new form without a catalog (versions default to latest)
    let yaml = "profiles:\n  dev:\n    tools: [nvim]\n";
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // assert
    assert_eq!(config.version_of("dev", "nvim"), "latest");
    assert!(config.has_tool_in("dev", "nvim"));
}

// --- Tools per profile ---

#[test]
fn tool_names_for_profile_returns_declared_order() {
    // arrange
    let yaml = "profiles:\n  dev:\n    tools: [tmux, nvim]\n  server:\n    tools: [tmux]\n";
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // act
    let names = config.tool_names_for_profile("dev");
    // assert
    assert_eq!(names, vec!["tmux", "nvim"]);
}

#[test]
fn tool_names_for_profile_is_empty_for_unknown() {
    // arrange
    let config: QwertConfig = QwertConfig::default();
    // act
    let names = config.tool_names_for_profile("ghost");
    // assert
    assert!(names.is_empty());
}

#[test]
fn profiles_are_independent() {
    // arrange
    let yaml = "profiles:\n  dev:\n    tools: [iterm2]\n  server:\n    tools: [nginx]\n";
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // act + assert
    assert!(!config.has_tool_in("dev", "nginx"));
    assert!(!config.has_tool_in("server", "iterm2"));
}

// --- Version & configs ---

#[test]
fn version_of_returns_catalog_version() {
    // arrange
    let yaml = "tools:\n  tmux: v3.4\nprofiles:\n  dev:\n    tools: [tmux]\n";
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // act
    let version = config.version_of("dev", "tmux");
    // assert
    assert_eq!(version, "v3.4");
}

#[test]
fn version_of_defaults_to_latest_when_undeclared() {
    // arrange
    let config: QwertConfig = QwertConfig::default();
    // act
    let version = config.version_of("dev", "missing");
    // assert
    assert_eq!(version, "latest");
}

#[test]
fn config_source_for_returns_declared_source() {
    // arrange
    let yaml = "profiles:\n  dev:\n    tools: [nvim]\n    configs:\n      nvim: ~/.qwert/nvim\n";
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // act
    let source = config.config_source_for("dev", "nvim");
    // assert
    assert_eq!(source, Some("~/.qwert/nvim"));
}

#[test]
fn config_source_for_missing_returns_none() {
    // arrange
    let config: QwertConfig = QwertConfig::default();
    // act + assert
    assert_eq!(config.config_source_for("dev", "nvim"), None);
}

#[test]
fn set_config_source_writes_and_overrides() {
    // arrange
    let mut config = QwertConfig::default();
    // act
    config.set_config_source("dev", "nvim", "~/.qwert/config/neovim");
    config.set_config_source("dev", "nvim", "~/.qwert/custom/neovim");
    // assert — latest write wins
    assert_eq!(
        config.config_source_for("dev", "nvim"),
        Some("~/.qwert/custom/neovim")
    );
}

// --- Mutations ---

#[test]
fn add_tool_adds_to_catalog_and_profile() {
    // arrange
    let mut config = QwertConfig::default();
    // act
    config.add_tool("dev", "nvim", None);
    // assert
    assert!(config.has_tool_in("dev", "nvim"));
    assert!(config.declared_anywhere("nvim"));
    assert_eq!(config.version_of("dev", "nvim"), "latest");
}

#[test]
fn add_tool_updates_version_in_catalog_only() {
    // arrange
    let mut config = QwertConfig::default();
    config.add_tool("dev", "nvim", Some("0.10"));
    // act
    config.add_tool("server", "nvim", None);
    // assert — catalog version stays at 0.10; both profiles reference it
    assert_eq!(config.version_of("dev", "nvim"), "0.10");
    assert!(config.has_tool_in("server", "nvim"));
}

#[test]
fn remove_tool_removes_from_catalog_and_profiles() {
    // arrange
    let mut config = QwertConfig::default();
    config.add_tool("dev", "tmux", None);
    config.add_tool("dev", "git", None);
    // act
    config.remove_tool("tmux");
    // assert
    assert!(!config.declared_anywhere("tmux"));
    assert!(config.declared_anywhere("git"));
}

#[test]
fn declared_anywhere_checks_catalog_and_profiles() {
    // arrange
    let yaml = "profiles:\n  server:\n    tools: [nginx]\n";
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // act + assert
    assert!(config.declared_anywhere("nginx"));
    assert!(!config.declared_anywhere("tmux"));
}

#[test]
fn profiles_of_tool_lists_all_declaring_profiles() {
    // arrange
    let yaml = "profiles:\n  dev:\n    tools: [nvim]\n  server:\n    tools: [nvim]\n";
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // act
    let profiles = config.profiles_of_tool("nvim");
    // assert
    assert_eq!(profiles, vec!["dev", "server"]);
}

// --- Hooks ---

#[test]
fn add_hook_appends_to_profile_and_dedups() {
    // arrange
    let mut config = QwertConfig::default();
    // act
    config.add_hook("dev", "init", "~/dev.sh");
    config.add_hook("dev", "init", "~/dev.sh");
    // assert
    assert_eq!(config.hooks_for_profile("dev", "init"), vec!["~/dev.sh"]);
}

#[test]
fn add_hook_ignores_unknown_hook() {
    // arrange
    let mut config = QwertConfig::default();
    // act
    config.add_hook("dev", "unknown", "~/x.sh");
    // assert
    assert!(config.hooks_for_profile("dev", "unknown").is_empty());
}

// --- Roundtrip & persistence ---

#[test]
fn save_and_load_roundtrip_with_profiles() {
    // arrange
    let yaml = "profiles:\n  dev:\n    tools: [tmux, nvim]\n    configs:\n      nvim: ~/.qwert/nvim\n    hooks:\n      init: [~/init.sh]\n  server:\n    tools: [tmux]\n";
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    let path = std::env::temp_dir().join("qwert_test_profiles_roundtrip.yml");
    // act
    config.save(&path).unwrap();
    let loaded = QwertConfig::load(&path).unwrap();
    fs::remove_file(&path).ok();
    // assert
    assert!(loaded.has_tool_in("dev", "nvim"));
    assert_eq!(loaded.config_source_for("dev", "nvim"), Some("~/.qwert/nvim"));
    assert_eq!(loaded.hooks_for_profile("dev", "init"), vec!["~/init.sh"]);
}

#[test]
fn load_returns_default_when_file_missing() {
    // arrange
    let path = std::env::temp_dir().join("qwert_nonexistent_xyz.yml");
    // act
    let config = QwertConfig::load(&path).unwrap();
    // assert
    assert!(config.profiles.is_empty());
}

// --- Inline setup (legacy catalog form) ---

#[test]
fn tool_entry_parses_string_and_object() {
    // arrange
    let yaml = "tools:\n  tmux: latest\n  neovim:\n    version: \"0.9\"\n";
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // assert
    assert_eq!(config.version_of("default", "tmux"), "latest");
    assert_eq!(config.version_of("default", "neovim"), "0.9");
}

#[test]
fn tool_entry_full_parses_from_object() {
    // arrange
    let yaml = "tools:\n  neovim:\n    version: \"0.9\"\n";
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // assert
    assert_eq!(config.version_of(PROFILE_DEFAULT, "neovim"), "0.9");
}

#[test]
fn inline_setup_of_returns_full_config_setup() {
    // arrange
    let yaml = "tools:\n  tmux:\n    version: latest\n    setup:\n      to: ~/.tmux.conf\n      symlink: true\n";
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    let setup: Option<&InlineSetup> = config.inline_setup_of("tmux");
    // assert
    assert!(setup.is_some());
    assert_eq!(setup.unwrap().to, "~/.tmux.conf");
}

#[test]
fn tool_entry_untagged_parses_simple() {
    // arrange
    let entry: crate::config::qwert_yml::ToolEntry =
        serde_yml::from_value(Value::String("latest".into())).unwrap();
    // act + assert
    match entry {
        crate::config::qwert_yml::ToolEntry::Simple(v) => assert_eq!(v, "latest"),
        _ => panic!("expected simple"),
    }
}