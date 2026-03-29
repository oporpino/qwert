use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use super::schema::Recipe;

/// Load a single recipe from a TOML file
pub fn load_recipe(path: &Path) -> Result<Recipe> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read recipe: {}", path.display()))?;
    toml::from_str(&content)
        .with_context(|| format!("failed to parse recipe: {}", path.display()))
}

/// Load all recipes from a directory
pub fn load_all(recipes_dir: &Path) -> Vec<Recipe> {
    let mut recipes = Vec::new();

    let Ok(entries) = std::fs::read_dir(recipes_dir) else {
        return recipes;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("toml")
            && path.file_name().and_then(|n| n.to_str()) != Some("stacks")
        {
            if let Ok(recipe) = load_recipe(&path) {
                recipes.push(recipe);
            }
        }
    }

    recipes.sort_by(|a, b| a.meta.name.cmp(&b.meta.name));
    recipes
}

/// Find a recipe by name in a directory
pub fn find(name: &str, recipes_dir: &Path) -> Option<Recipe> {
    let path = recipes_dir.join(format!("{}.toml", name));
    load_recipe(&path).ok()
}

/// Path to the recipes cache directory (~/.qwert/recipes/)
pub fn cache_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".qwert").join("recipes"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const MINIMAL_RECIPE: &str = r#"
[meta]
name = "tmux"
version = "1.0.0"
description = "Terminal multiplexer"
type = "brew"

[check]
command = "tmux"

[install]
macos = "brew install tmux"
"#;

    #[test]
    fn load_recipe_parses_valid_toml() {
        // arrange
        let path = std::env::temp_dir().join("test_tmux.toml");
        fs::write(&path, MINIMAL_RECIPE).unwrap();
        // act
        let recipe = load_recipe(&path).unwrap();
        fs::remove_file(&path).ok();
        // assert
        assert_eq!(recipe.meta.name, "tmux");
        assert_eq!(recipe.meta.version, "1.0.0");
    }

    #[test]
    fn load_recipe_errors_on_missing_file() {
        // arrange
        let path = std::path::Path::new("/tmp/nonexistent_recipe_xyz.toml");
        // act
        let result = load_recipe(path);
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn load_recipe_errors_on_invalid_toml() {
        // arrange
        let path = std::env::temp_dir().join("bad_recipe.toml");
        fs::write(&path, "this is not valid toml [[[").unwrap();
        // act
        let result = load_recipe(&path);
        fs::remove_file(&path).ok();
        // assert
        assert!(result.is_err());
    }

    #[test]
    fn find_returns_recipe_by_name() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_test_recipes");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("tmux.toml"), MINIMAL_RECIPE).unwrap();
        // act
        let result = find("tmux", &dir);
        fs::remove_dir_all(&dir).ok();
        // assert
        assert!(result.is_some());
        assert_eq!(result.unwrap().meta.name, "tmux");
    }

    #[test]
    fn find_returns_none_when_recipe_missing() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_test_empty_recipes");
        fs::create_dir_all(&dir).unwrap();
        // act
        let result = find("neovim", &dir);
        fs::remove_dir_all(&dir).ok();
        // assert
        assert!(result.is_none());
    }

    #[test]
    fn load_all_returns_recipes_sorted_by_name() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_test_load_all");
        fs::create_dir_all(&dir).unwrap();
        let tmux = MINIMAL_RECIPE;
        let neovim = tmux.replace("name = \"tmux\"", "name = \"neovim\"")
            .replace("command = \"tmux\"", "command = \"nvim\"")
            .replace("brew install tmux", "brew install neovim");
        fs::write(dir.join("tmux.toml"), tmux).unwrap();
        fs::write(dir.join("neovim.toml"), neovim).unwrap();
        // act
        let recipes = load_all(&dir);
        fs::remove_dir_all(&dir).ok();
        // assert
        assert_eq!(recipes.len(), 2);
        assert_eq!(recipes[0].meta.name, "neovim");
        assert_eq!(recipes[1].meta.name, "tmux");
    }

    #[test]
    fn load_all_returns_empty_for_missing_dir() {
        // arrange
        let dir = std::path::Path::new("/tmp/qwert_definitely_missing_dir");
        // act
        let recipes = load_all(dir);
        // assert
        assert!(recipes.is_empty());
    }
}
