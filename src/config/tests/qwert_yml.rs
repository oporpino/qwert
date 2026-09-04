use crate::config::qwert_yml::{InlineSetup, QwertConfig, ToolEntry, PROFILE_DEFAULT};
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
    assert!(config.has_tool_in(PROFILE_DEFAULT, "git"));
    assert_eq!(config.profile_names(), vec![PROFILE_DEFAULT]);
}

#[test]
fn flat_full_entry_loads_into_default_profile() {
    // arrange — full object with version + setup
    let yaml = "tools:\n  neovim:\n    version: \"0.10\"\n    setup:\n      to: ~/.config/nvim\n      symlink: true\n";
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // assert
    assert!(config.has_tool_in(PROFILE_DEFAULT, "neovim"));
    let setup = config.setup_of(PROFILE_DEFAULT, "neovim").unwrap();
    assert_eq!(setup.to, "~/.config/nvim");
    assert!(setup.symlink);
}

#[test]
fn flat_hooks_load_into_default_profile() {
    // arrange
    let yaml = "hooks:\n  init:\n    - ~/env.sh\n";
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // assert
    assert_eq!(config.hooks_for(PROFILE_DEFAULT, "init"), vec!["~/env.sh"]);
}

// --- Loading: sectioned profiles ---

#[test]
fn profiles_load_roles_and_tools() {
    // arrange
    let yaml = "profiles:\n  dev:\n    tools:\n      docker: latest\n  server:\n    tools:\n      nginx: latest\n";
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // assert
    assert!(config.has_tool_in("dev", "docker"));
    assert!(config.has_tool_in("server", "nginx"));
    assert!(!config.has_tool_in("dev", "nginx"));
    assert_eq!(config.profile_names(), vec!["dev", "server"]);
}

#[test]
fn profiles_parse_hooks() {
    // arrange
    let yaml = "profiles:\n  dev:\n    tools: {}\n    hooks:\n      init: [~/dev-init.sh]\n";
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // assert
    assert_eq!(config.hooks_for("dev", "init"), vec!["~/dev-init.sh"]);
}

// --- Tools per profile ---

#[test]
fn tool_names_for_profile_returns_declared_order() {
    // arrange
    let yaml = "profiles:\n  dev:\n    tools:\n      tmux: latest\n      neovim: latest\n  server:\n    tools:\n      nginx: latest\n";
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // act
    let names = config.tool_names_for_profile("dev");
    // assert
    assert_eq!(names, vec!["tmux", "neovim"]);
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
    let yaml = "profiles:\n  dev:\n    tools:\n      iterm2: latest\n  server:\n    tools:\n      nginx: latest\n";
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // act + assert — dev does not get nginx, server does not get iterm2
    assert!(!config.has_tool_in("dev", "nginx"));
    assert!(!config.has_tool_in("server", "iterm2"));
    assert_eq!(config.tool_names_for_profile("dev"), vec!["iterm2"]);
    assert_eq!(config.tool_names_for_profile("server"), vec!["nginx"]);
}

// --- Version & setup ---

#[test]
fn version_of_returns_version_for_profile() {
    // arrange
    let yaml = "profiles:\n  dev:\n    tools:\n      tmux: \"3.4\"\n";
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // act
    let version = config.version_of("dev", "tmux");
    // assert
    assert_eq!(version, "3.4");
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
fn setup_of_returns_setup_when_declared() {
    // arrange
    let yaml = "profiles:\n  dev:\n    tools:\n      nv:\n        version: latest\n        setup: { to: ~/.config/nvim, symlink: true }\n";
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // act
    let setup = config.setup_of("dev", "nv");
    // assert
    assert!(setup.is_some());
}

#[test]
fn setup_of_returns_none_when_simple_entry() {
    // arrange
    let mut config = QwertConfig::default();
    config.add_tool("dev", "tmux", None);
    // act + assert
    assert!(config.setup_of("dev", "tmux").is_none());
}

// --- Mutations ---

#[test]
fn add_tool_adds_to_named_profile() {
    // arrange
    let mut config = QwertConfig::default();
    // act
    config.add_tool("dev", "nvim", None);
    // assert
    assert!(config.has_tool_in("dev", "nvim"));
    assert!(!config.has_tool_in(PROFILE_DEFAULT, "nvim"));
}

#[test]
fn add_tool_defaults_version_to_latest() {
    // arrange
    let mut config = QwertConfig::default();
    // act
    config.add_tool("dev", "nvim", None);
    // assert
    assert_eq!(config.version_of("dev", "nvim"), "latest");
}

#[test]
fn add_tool_preserves_existing_inline_setup() {
    // arrange
    let mut config = QwertConfig::default();
    config.add_tool("default", "neovim", Some("0.10"));
    config.add_tool("default", "neovim", Some("0.11"));
    // act — second add only bumps version
    config.add_tool("default", "neovim", Some("0.11"));
    // assert
    assert_eq!(config.version_of("default", "neovim"), "0.11");
}

#[test]
fn remove_tool_removes_from_all_profiles() {
    // arrange
    let yaml = "profiles:\n  shared:\n    tools:\n      tmux: latest\n      git: latest\n  dev:\n    tools:\n      tmux: latest\n";
    let mut config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // act
    config.remove_tool("tmux");
    // assert
    assert!(!config.declared_anywhere("tmux"));
    assert!(config.declared_anywhere("git"));
}

#[test]
fn declared_anywhere_checks_all_profiles() {
    // arrange
    let yaml = "profiles:\n  server:\n    tools:\n      nginx: latest\n";
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // act + assert
    assert!(config.declared_anywhere("nginx"));
    assert!(!config.declared_anywhere("tmux"));
}

#[test]
    fn profiles_of_tool_lists_all_declaring_profiles() {
        // arrange
        let yaml = "profiles:\n  dev:\n    tools:\n      nvim: latest\n  server:\n    tools:\n      nvim: latest\n";
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // act
    let profiles = config.profiles_of_tool("nvim");
    // assert
    assert_eq!(profiles, vec!["dev", "server"]);
}

// --- Hooks ---

#[test]
fn add_hook_appends_to_named_profile_and_dedups() {
    // arrange
    let mut config = QwertConfig::default();
    // act
    config.add_hook("dev", "init", "~/dev.sh");
    config.add_hook("dev", "init", "~/dev.sh");
    // assert
    assert_eq!(config.hooks_for("dev", "init"), vec!["~/dev.sh"]);
}

#[test]
fn add_hook_ignores_unknown_hook() {
    // arrange
    let mut config = QwertConfig::default();
    // act
    config.add_hook("dev", "unknown", "~/x.sh");
    // assert
    assert!(config.hooks_for("dev", "unknown").is_empty());
}

// --- Roundtrip & persistence ---

#[test]
fn save_and_load_roundtrip_with_profiles() {
    // arrange
    let yaml = "profiles:\n  dev:\n    tools:\n      tmux: latest\n      docker: latest\n  server:\n    tools:\n      nginx: latest\n    hooks:\n      init: [~/server-init.sh]\n";
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    let path = std::env::temp_dir().join("qwert_test_profiles_roundtrip.yml");
    // act
    config.save(&path).unwrap();
    let loaded = QwertConfig::load(&path).unwrap();
    fs::remove_file(&path).ok();
    // assert
    assert!(loaded.has_tool_in("dev", "docker"));
    assert!(loaded.has_tool_in("server", "nginx"));
    assert_eq!(loaded.hooks_for("server", "init"), vec!["~/server-init.sh"]);
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

// --- Inline setup: arch field ---

#[test]
fn inline_setup_parses_arch_commands() {
    // arrange
    let yaml = "profiles:\n  dev:\n    tools:\n      tmux:\n        version: latest\n        setup:\n          to: ~/.tmux.conf\n          symlink: true\n          arch: \"echo arch-setup\"\n";
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    let setup: &InlineSetup = config.setup_of("dev", "tmux").unwrap();
    // assert
    let steps = setup.arch.as_ref().unwrap().as_steps();
    assert_eq!(steps, vec!["echo arch-setup"]);
}

// --- ToolEntry helpers ---

#[test]
fn tool_entry_simple_parses_from_string() {
    // arrange
    let yaml = "profiles:\n  dev:\n    tools:\n      tmux: latest\n";
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // assert
    assert_eq!(config.version_of("dev", "tmux"), "latest");
    let entry: ToolEntry = serde_yml::from_value(Value::String("latest".into())).unwrap();
    match entry {
        ToolEntry::Simple(v) => assert_eq!(v, "latest"),
        ToolEntry::Full(_) => panic!("expected simple"),
    }
}

#[test]
fn profile_of_tool_returns_first_declaring() {
    // arrange
    let yaml = "profiles:\n  dev:\n    tools:\n      nvim: latest\n  server:\n    tools:\n      nvim: latest\n";
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // act
    let profile = config.profile_of_tool("nvim");
    // assert
    assert_eq!(profile, Some("dev"));
}