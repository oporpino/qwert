use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};

use super::schema::{InstallFile, Recipe, RecipeCheck, RecipeMeta, RecipeKind, RecipeSetup, SetupFile};
use crate::platform;

fn load_toml_opt<T: DeserializeOwned>(path: &Path) -> Option<T> {
    let content = std::fs::read_to_string(path).ok()?;
    toml::from_str(&content).ok()
}

fn default_kind() -> RecipeKind {
    match platform::detect() {
        platform::Platform::MacOS => RecipeKind::Brew,
        platform::Platform::Arch => RecipeKind::Pacman,
        platform::Platform::Debian | platform::Platform::Unknown => RecipeKind::Apt,
    }
}

fn assemble_recipe(name: &str, install: Option<InstallFile>, setup: Option<SetupFile>) -> Recipe {
    let meta = install.as_ref().map(|i| i.meta.clone()).unwrap_or_else(|| RecipeMeta {
        name: name.to_string(),
        version: String::new(),
        description: String::new(),
        kind: default_kind(),
        depends: vec![],
        pkg: None,
    });

    let check = install.as_ref().and_then(|i| i.check.clone()).or_else(|| {
        Some(RecipeCheck { command: name.to_string(), version_flag: None })
    });

    let recipe_setup = setup.map(|s| RecipeSetup {
        from: s.from,
        to: s.to,
        symlink: s.symlink,
        macos: s.macos,
        debian: s.debian,
        undo: s.undo,
    });

    Recipe {
        meta,
        check,
        install: install.as_ref().and_then(|i| i.install.clone()),
        upgrade: install.as_ref().and_then(|i| i.upgrade.clone()),
        uninstall: install.as_ref().and_then(|i| i.uninstall.clone()),
        setup: recipe_setup,
    }
}

/// Find a recipe by name — loads from <recipes_dir>/<name>/install.toml and/or setup.toml
pub fn find(name: &str, recipes_dir: &Path) -> Option<Recipe> {
    let dir = recipes_dir.join(name);
    let install: Option<InstallFile> = load_toml_opt(&dir.join("install.toml"));
    let setup: Option<SetupFile> = load_toml_opt(&dir.join("setup.toml"));
    if install.is_none() && setup.is_none() {
        return None;
    }
    Some(assemble_recipe(name, install, setup))
}

/// Load all recipes from subdirectories
pub fn load_all(recipes_dir: &Path) -> Vec<Recipe> {
    let mut recipes = Vec::new();

    let Ok(entries) = std::fs::read_dir(recipes_dir) else {
        return recipes;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if let Some(recipe) = find(name, recipes_dir) {
                recipes.push(recipe);
            }
        }
    }

    recipes.sort_by(|a, b| a.meta.name.cmp(&b.meta.name));
    recipes
}

/// Path to the recipes cache directory (~/.local/share/qwert/recipes/)
pub fn cache_dir() -> Option<PathBuf> {
    Some(crate::platform::data_dir().join("recipes"))
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const INSTALL_TOML: &str = r#"
[meta]
name = "tmux"
version = "1.0.0"
description = "Terminal multiplexer"
type = "brew"

[check]
command = "tmux"
version_flag = "-V"
"#;

    const SETUP_TOML: &str = r#"
to = "~/.tmux.conf"
symlink = true
"#;

    fn make_recipe_dir(dir: &Path, name: &str, install: Option<&str>, setup: Option<&str>) {
        let recipe_dir = dir.join(name);
        fs::create_dir_all(&recipe_dir).unwrap();
        if let Some(content) = install {
            fs::write(recipe_dir.join("install.toml"), content).unwrap();
        }
        if let Some(content) = setup {
            fs::write(recipe_dir.join("setup.toml"), content).unwrap();
        }
    }

    #[test]
    fn find_returns_recipe_from_directory() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_test_find_dir");
        make_recipe_dir(&dir, "tmux", Some(INSTALL_TOML), None);
        // act
        let result = find("tmux", &dir);
        // assert
        assert!(result.is_some());
        assert_eq!(result.unwrap().meta.name, "tmux");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn find_returns_none_when_no_directory() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_test_find_none");
        fs::create_dir_all(&dir).unwrap();
        // act
        let result = find("neovim", &dir);
        // assert
        assert!(result.is_none());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn find_assembles_install_and_setup_when_both_present() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_test_find_both");
        make_recipe_dir(&dir, "tmux", Some(INSTALL_TOML), Some(SETUP_TOML));
        // act
        let result = find("tmux", &dir).unwrap();
        // assert
        assert!(result.setup.is_some());
        assert_eq!(result.setup.unwrap().to, "~/.tmux.conf");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn find_returns_setup_none_when_only_install_toml() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_test_find_install_only");
        make_recipe_dir(&dir, "tmux", Some(INSTALL_TOML), None);
        // act
        let result = find("tmux", &dir).unwrap();
        // assert
        assert!(result.setup.is_none());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn find_synthesizes_meta_when_only_setup_toml() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_test_find_setup_only");
        make_recipe_dir(&dir, "tmux", None, Some(SETUP_TOML));
        // act
        let result = find("tmux", &dir).unwrap();
        // assert
        assert_eq!(result.meta.name, "tmux");
        assert!(result.setup.is_some());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn find_synthesizes_check_when_no_install_toml() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_test_find_check_synth");
        make_recipe_dir(&dir, "git", None, Some(SETUP_TOML));
        // act
        let result = find("git", &dir).unwrap();
        // assert
        let check = result.check.unwrap();
        assert_eq!(check.command, "git");
        assert!(check.version_flag.is_none());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_all_returns_recipes_sorted_by_name_from_subdirs() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_test_load_all_dirs");
        let neovim_install = INSTALL_TOML.replace("name = \"tmux\"", "name = \"neovim\"")
            .replace("command = \"tmux\"", "command = \"nvim\"");
        make_recipe_dir(&dir, "tmux", Some(INSTALL_TOML), None);
        make_recipe_dir(&dir, "neovim", Some(&neovim_install), None);
        // act
        let recipes = load_all(&dir);
        // assert
        assert_eq!(recipes.len(), 2);
        assert_eq!(recipes[0].meta.name, "neovim");
        assert_eq!(recipes[1].meta.name, "tmux");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_all_returns_empty_for_missing_dir() {
        // arrange
        let dir = std::path::Path::new("/tmp/qwert_definitely_missing_dir_xyz");
        // act
        let recipes = load_all(dir);
        // assert
        assert!(recipes.is_empty());
    }
}
