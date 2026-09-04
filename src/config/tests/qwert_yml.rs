use crate::config::qwert_yml::{InlineSetup, QwertConfig, SHARED};
use std::fs;

// --- Loading: backward-compatible flat format ---

#[test]
fn flat_tools_load_into_shared_section() {
    // arrange
    let yaml = "tools:\n  tmux: latest\n  git: latest\n";
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // assert
    assert!(config.has_tool_in(SHARED, "tmux"));
    assert!(config.has_tool_in(SHARED, "git"));
    assert!(config.role_sections().is_empty());
}

#[test]
fn flat_full_entry_loads_into_shared_section() {
    // arrange — full object with version + setup must still be treated as flat
    let yaml = "tools:\n  neovim:\n    version: \"0.10\"\n    setup:\n      to: ~/.config/nvim\n      symlink: true\n";
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // assert
    assert!(config.has_tool_in(SHARED, "neovim"));
    let setup = config.setup_of_for_roles("neovim", &[]).unwrap();
    assert_eq!(setup.to, "~/.config/nvim");
    assert!(setup.symlink);
}

// --- Loading: sectioned format ---

#[test]
fn sectioned_tools_load_roles() {
    // arrange
    let yaml = "tools:\n  shared:\n    tmux: latest\n  dev:\n    docker: latest\n  server:\n    nginx: latest\n";
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // assert
    assert!(config.has_tool_in("shared", "tmux"));
    assert!(config.has_tool_in("dev", "docker"));
    assert!(config.has_tool_in("server", "nginx"));
    assert_eq!(config.role_sections(), vec!["dev", "server"]);
}

// --- Effective sections & union ---

#[test]
fn effective_sections_puts_shared_first() {
    // arrange
    let config = QwertConfig::default();
    // act
    let sections = config.effective_sections(&["server".into(), "dev".into()]);
    // assert
    assert_eq!(sections, vec!["shared", "server", "dev"]);
}

#[test]
fn effective_sections_dedups_and_skips_shared() {
    // arrange
    let config = QwertConfig::default();
    // act
    let sections = config.effective_sections(&["shared".into(), "dev".into(), "dev".into()]);
    // assert
    assert_eq!(sections, vec!["shared", "dev"]);
}

#[test]
fn tool_names_for_roles_unions_sections_in_order() {
    // arrange
    let yaml = "tools:\n  shared:\n    tmux: latest\n  dev:\n    docker: latest\n    nvim: latest\n  server:\n    nginx: latest\n    docker: latest\n";
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // act
    let names = config.tool_names_for_roles(&["dev".into(), "server".into()]);
    // assert — docker deduped, nginx after dev's tools
    assert_eq!(names, vec!["tmux", "docker", "nvim", "nginx"]);
}

#[test]
fn tool_names_for_roles_excludes_inactive_roles() {
    // arrange
    let yaml = "tools:\n  shared:\n    tmux: latest\n  server:\n    nginx: latest\n";
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // act
    let names = config.tool_names_for_roles(&["dev".into()]);
    // assert — server is not active
    assert_eq!(names, vec!["tmux"]);
}

// --- Last-wins semantics ---

#[test]
fn version_last_declared_role_wins() {
    // arrange
    let yaml = "tools:\n  shared:\n    nvim: \"0.9\"\n  dev:\n    nvim: \"0.10\"\n";
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // act + assert
    assert_eq!(config.version_of_for_roles("nvim", &["dev".into()]), "0.10");
    assert_eq!(config.version_of_for_roles("nvim", &[]), "0.9");
}

#[test]
fn version_defaults_to_latest_when_undeclared() {
    // arrange
    let config = QwertConfig::default();
    // act
    let version = config.version_of_for_roles("missing", &[]);
    // assert
    assert_eq!(version, "latest");
}

#[test]
fn setup_of_for_roles_picks_last_declared_section() {
    // arrange
    let yaml = r#"
tools:
  shared:
    nvim:
      version: latest
      setup: { to: ~/.config/nvim-shared, symlink: true }
  server:
    nvim:
      version: latest
      setup: { to: ~/.config/nvim-server, symlink: true }
"#;
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // act
    let setup = config.setup_of_for_roles("nvim", &["server".into()]).unwrap();
    // assert
    assert_eq!(setup.to, "~/.config/nvim-server");
}

// --- Mutations ---

#[test]
fn add_tool_adds_to_named_role() {
    // arrange
    let mut config = QwertConfig::default();
    // act
    config.add_tool("nvim", "dev", None);
    // assert
    assert!(config.has_tool_in("dev", "nvim"));
    assert!(!config.has_tool_in(SHARED, "nvim"));
    assert_eq!(config.role_sections(), vec!["dev"]);
}

#[test]
fn add_tool_defaults_version_to_latest() {
    // arrange
    let mut config = QwertConfig::default();
    // act
    config.add_tool("nvim", "dev", None);
    // assert
    assert_eq!(config.version_of_for_roles("nvim", &["dev".into()]), "latest");
}

#[test]
fn add_tool_preserves_existing_inline_setup() {
    // arrange
    let yaml = "tools:\n  neovim:\n    version: latest\n    setup:\n      to: ~/.config/nvim\n      symlink: true\n";
    let mut config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // act — update version without touching setup
    config.add_tool("neovim", SHARED, Some("0.10"));
    // assert
    assert_eq!(config.version_of_for_roles("neovim", &[]), "0.10");
    assert!(config.setup_of_for_roles("neovim", &[]).is_some());
}

#[test]
fn remove_tool_removes_from_all_sections_and_drops_empty_role() {
    // arrange
    let yaml = "tools:\n  shared:\n    tmux: latest\n    git: latest\n  dev:\n    tmux: latest\n";
    let mut config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // act
    config.remove_tool("tmux");
    // assert
    assert!(!config.declared_anywhere("tmux"));
    assert!(config.declared_anywhere("git"));
    assert!(config.role_sections().is_empty());
}

#[test]
fn declared_anywhere_checks_all_sections() {
    // arrange
    let yaml = "tools:\n  server:\n    nginx: latest\n";
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // act + assert
    assert!(config.declared_anywhere("nginx"));
    assert!(!config.declared_anywhere("tmux"));
}

// --- Hooks ---

#[test]
fn flat_hooks_load_into_shared() {
    // arrange
    let yaml = "hooks:\n  init:\n    - ~/env.sh\n";
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // assert
    assert_eq!(config.hooks.get(SHARED).unwrap().init, vec!["~/env.sh"]);
}

#[test]
fn sectioned_hooks_load_by_role() {
    // arrange
    let yaml = "hooks:\n  shared:\n    init: [~/common.sh]\n  dev:\n    prepare: [~/dev.sh]\n";
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // assert
    assert_eq!(config.hooks.get("shared").unwrap().init, vec!["~/common.sh"]);
    assert_eq!(config.hooks.get("dev").unwrap().prepare, vec!["~/dev.sh"]);
}

#[test]
fn add_hook_appends_to_named_role_and_dedups() {
    // arrange
    let mut config = QwertConfig::default();
    // act
    config.add_hook("dev", "init", "~/dev.sh");
    config.add_hook("dev", "init", "~/dev.sh");
    // assert
    assert_eq!(config.hooks.get("dev").unwrap().init, vec!["~/dev.sh"]);
}

#[test]
fn add_hook_ignores_unknown_hook() {
    // arrange
    let mut config = QwertConfig::default();
    // act
    config.add_hook("dev", "unknown", "~/x.sh");
    // assert
    assert!(!config.hooks.contains_key("dev"));
}

// --- Roundtrip & persistence ---

#[test]
fn save_and_load_roundtrip_with_sections() {
    // arrange
    let yaml = "tools:\n  shared:\n    tmux: latest\n  dev:\n    docker: latest\nhooks:\n  shared:\n    init: [~/env.sh]\n";
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    let path = std::env::temp_dir().join("qwert_test_sections_roundtrip.yml");
    // act
    config.save(&path).unwrap();
    let loaded = QwertConfig::load(&path).unwrap();
    fs::remove_file(&path).ok();
    // assert
    assert!(loaded.has_tool_in("dev", "docker"));
    assert!(loaded.has_tool_in(SHARED, "tmux"));
    assert_eq!(loaded.hooks.get(SHARED).unwrap().init, vec!["~/env.sh"]);
}

#[test]
fn save_drops_empty_sections() {
    // arrange — dev section becomes empty after removing its only tool
    let mut config = QwertConfig::default();
    config.add_tool("tmux", "dev", None);
    config.add_tool("git", SHARED, None);
    config.remove_tool("tmux");
    let path = std::env::temp_dir().join("qwert_test_empty_section.yml");
    // act
    config.save(&path).unwrap();
    let content = fs::read_to_string(&path).unwrap();
    fs::remove_file(&path).ok();
    // assert — empty dev section not serialized, shared tool kept
    assert!(!content.contains("dev:"));
    assert!(content.contains("git"));
}

#[test]
fn load_returns_default_when_file_missing() {
    // arrange
    let path = std::env::temp_dir().join("qwert_nonexistent_xyz.yml");
    // act
    let config = QwertConfig::load(&path).unwrap();
    // assert
    assert!(config.tools.is_empty());
}

// --- Inline setup: arch field ---

#[test]
fn inline_setup_parses_arch_commands() {
    // arrange
    let yaml = "tools:\n  tmux:\n    version: latest\n    setup:\n      to: ~/.tmux.conf\n      symlink: true\n      arch: \"echo arch-setup\"\n";
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    let setup: &InlineSetup = config.setup_of_for_roles("tmux", &[]).unwrap();
    // assert
    let steps = setup.arch.as_ref().unwrap().as_steps();
    assert_eq!(steps, vec!["echo arch-setup"]);
}

#[test]
fn inline_setup_parses_macos_debian_and_arch() {
    // arrange
    let yaml = r#"
tools:
  delta:
    version: latest
    setup:
      to: ~/.gitconfig
      macos: "git config --global core.pager delta"
      debian: "git config --global core.pager delta"
      arch: "git config --global core.pager delta"
"#;
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    let setup: &InlineSetup = config.setup_of_for_roles("delta", &[]).unwrap();
    // assert
    assert!(setup.macos.is_some());
    assert!(setup.debian.is_some());
    assert!(setup.arch.is_some());
}

// --- ToolEntry helpers used elsewhere ---

#[test]
fn tool_entry_simple_parses_from_string() {
    // arrange
    let yaml = "tools:\n  tmux: latest\n";
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // assert
    assert_eq!(config.version_of_for_roles("tmux", &[]), "latest");
}

#[test]
fn tool_entry_full_parses_from_object() {
    // arrange
    let yaml = "tools:\n  neovim:\n    version: \"0.9\"\n";
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // assert
    assert_eq!(config.version_of_for_roles("neovim", &[]), "0.9");
}

#[test]
fn sections_of_tool_lists_all_declaring_sections() {
    // arrange
    let yaml = "tools:\n  shared:\n    nvim: latest\n  dev:\n    nvim: latest\n  server:\n    nginx: latest\n";
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // act
    let sections = config.sections_of_tool("nvim");
    // assert
    assert_eq!(sections, vec!["shared", "dev"]);
}