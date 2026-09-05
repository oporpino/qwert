use crate::config::qwert_yml::{PluginSource, QwertConfig};
use crate::plugins::derive_name;

// --- derive_name ---

#[test]
fn derive_name_from_https_url() {
    // arrange
    let url = "https://github.com/br4zz4/qwert-recipes-neovim";
    // act
    let name = derive_name(url).unwrap();
    // assert
    assert_eq!(name, "qwert-recipes-neovim");
}

#[test]
fn derive_name_strips_dot_git_and_trailing_slash() {
    // arrange
    let url = "https://github.com/myorg/my-recipes.git/";
    // act
    let name = derive_name(url).unwrap();
    // assert
    assert_eq!(name, "my-recipes");
}

#[test]
fn derive_name_from_ssh_scp_url() {
    // arrange
    let url = "git@github.com:myorg/some-recipes.git";
    // act
    let name = derive_name(url).unwrap();
    // assert
    assert_eq!(name, "some-recipes");
}

#[test]
fn derive_name_from_local_path() {
    // arrange
    let url = "/Users/dev/labs/custom-recipes";
    // act
    let name = derive_name(url).unwrap();
    // assert
    assert_eq!(name, "custom-recipes");
}

#[test]
fn derive_name_rejects_invalid_characters() {
    // arrange
    let url = "https://github.com/br4zz4/qwert recipes";
    // act
    let result = derive_name(url);
    // assert
    assert!(result.is_err());
}

#[test]
fn derive_name_rejects_empty_url() {
    // arrange
    let url = "https://github.com/";
    // act
    let result = derive_name(url);
    // assert
    assert!(result.is_err());
}

// --- config plugin declarations ---

#[test]
fn add_plugin_records_name_and_url() {
    // arrange
    let mut config = QwertConfig::default();
    // act
    config.add_plugin("my-recipes", "https://github.com/x/my-recipes");
    // assert
    let plugins: Vec<&PluginSource> = config.plugins().iter().collect();
    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0].name, "my-recipes");
    assert_eq!(plugins[0].url, "https://github.com/x/my-recipes");
}

#[test]
fn add_plugin_replaces_same_name_url() {
    // arrange
    let mut config = QwertConfig::default();
    config.add_plugin("my-recipes", "https://github.com/x/my-recipes");
    // act
    config.add_plugin("my-recipes", "https://github.com/y/my-recipes");
    // assert
    assert_eq!(config.plugins().len(), 1);
    assert_eq!(config.plugins()[0].url, "https://github.com/y/my-recipes");
}

#[test]
fn remove_plugin_returns_whether_declared() {
    // arrange
    let mut config = QwertConfig::default();
    config.add_plugin("a-recipes", "https://github.com/x/a-recipes");
    // act
    let removed = config.remove_plugin("a-recipes");
    let missing = config.remove_plugin("ghost-recipes");
    // assert
    assert!(removed);
    assert!(!missing);
    assert!(config.plugins().is_empty());
}

#[test]
fn plugins_roundtrip_through_serde() {
    // arrange
    let yaml = "plugins:\n  - name: my-recipes\n    url: https://github.com/x/my-recipes\n";
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    let path = std::env::temp_dir().join("qwert_test_plugins_roundtrip.yml");
    // act
    config.save(&path).unwrap();
    let loaded = QwertConfig::load(&path).unwrap();
    std::fs::remove_file(&path).ok();
    // assert
    assert_eq!(loaded.plugins().len(), 1);
    assert_eq!(loaded.plugins()[0].name, "my-recipes");
    assert_eq!(loaded.plugins()[0].url, "https://github.com/x/my-recipes");
}

#[test]
fn config_without_plugins_defaults_empty() {
    // arrange
    let yaml = "tools:\n  tmux: latest\n";
    // act
    let config: QwertConfig = serde_yml::from_str(yaml).unwrap();
    // assert
    assert!(config.plugins().is_empty());
}