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

/// Path to the bundled recipes (relative to the installed qwert dir)
pub fn bundled_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".qwert").join("recipes"))
}
