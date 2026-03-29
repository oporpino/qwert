use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Recipe {
    pub meta: RecipeMeta,
    pub check: Option<RecipeCheck>,
    pub install: Option<RecipeInstall>,
    pub upgrade: Option<RecipeUpgrade>,
    pub config: Option<RecipeConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RecipeMeta {
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(rename = "type")]
    pub kind: RecipeKind,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RecipeKind {
    Brew,
    Qwert,
}

impl std::fmt::Display for RecipeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecipeKind::Brew => write!(f, "brew"),
            RecipeKind::Qwert => write!(f, "qwert"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RecipeCheck {
    /// Binary name to check with `which`
    pub command: String,
    /// Optional flag to get version string (e.g. "--version")
    pub version_flag: Option<String>,
}

/// A single command string or an ordered list of commands.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Commands {
    One(String),
    Many(Vec<String>),
}

impl Commands {
    pub fn as_steps(&self) -> Vec<&str> {
        match self {
            Commands::One(s) => vec![s.as_str()],
            Commands::Many(v) => v.iter().map(|s| s.as_str()).collect(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RecipeInstall {
    pub macos: Option<Commands>,
    /// Debian-based Linux (Ubuntu, Debian, etc.)
    pub debian: Option<Commands>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RecipeUpgrade {
    pub macos: Option<Commands>,
    pub debian: Option<Commands>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RecipeConfig {
    /// Path relative to qwert installation dir
    pub src: Option<String>,
    /// Destination path (supports ~)
    pub dest: String,
    /// Create a symlink instead of copying
    #[serde(default)]
    pub symlink: bool,
}

fn make_recipe(install_macos: Option<Commands>, install_debian: Option<Commands>) -> Recipe {
    Recipe {
        meta: RecipeMeta {
            name: "test".into(),
            version: "1.0.0".into(),
            description: "test recipe".into(),
            kind: RecipeKind::Brew,
        },
        check: None,
        install: Some(RecipeInstall { macos: install_macos, debian: install_debian }),
        upgrade: None,
        config: None,
    }
}

impl Recipe {
    pub fn install_steps_for(&self, platform: &crate::platform::Platform) -> Vec<&str> {
        let Some(install) = &self.install else { return vec![] };
        let cmds = match platform {
            crate::platform::Platform::MacOS => install.macos.as_ref(),
            crate::platform::Platform::Debian | crate::platform::Platform::Unknown => install.debian.as_ref(),
        };
        cmds.map(|c| c.as_steps()).unwrap_or_default()
    }

    pub fn upgrade_steps_for(&self, platform: &crate::platform::Platform) -> Vec<&str> {
        let Some(upgrade) = &self.upgrade else { return vec![] };
        let cmds = match platform {
            crate::platform::Platform::MacOS => upgrade.macos.as_ref(),
            crate::platform::Platform::Debian | crate::platform::Platform::Unknown => upgrade.debian.as_ref(),
        };
        cmds.map(|c| c.as_steps()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::Platform;

    #[test]
    fn commands_one_returns_single_step() {
        // arrange
        let cmd = Commands::One("brew install tmux".into());
        // act
        let steps = cmd.as_steps();
        // assert
        assert_eq!(steps, vec!["brew install tmux"]);
    }

    #[test]
    fn commands_many_returns_all_steps_in_order() {
        // arrange
        let cmd = Commands::Many(vec!["step one".into(), "step two".into(), "step three".into()]);
        // act
        let steps = cmd.as_steps();
        // assert
        assert_eq!(steps, vec!["step one", "step two", "step three"]);
    }

    #[test]
    fn install_steps_for_macos_returns_macos_commands() {
        // arrange
        let recipe = make_recipe(
            Some(Commands::One("brew install nvim".into())),
            Some(Commands::One("apt install nvim".into())),
        );
        // act
        let steps = recipe.install_steps_for(&Platform::MacOS);
        // assert
        assert_eq!(steps, vec!["brew install nvim"]);
    }

    #[test]
    fn install_steps_for_debian_returns_debian_commands() {
        // arrange
        let recipe = make_recipe(
            Some(Commands::One("brew install nvim".into())),
            Some(Commands::One("apt install nvim".into())),
        );
        // act
        let steps = recipe.install_steps_for(&Platform::Debian);
        // assert
        assert_eq!(steps, vec!["apt install nvim"]);
    }

    #[test]
    fn install_steps_empty_when_platform_not_supported() {
        // arrange — macos-only recipe, queried for Debian
        let recipe = make_recipe(Some(Commands::One("brew install nvim".into())), None);
        // act
        let steps = recipe.install_steps_for(&Platform::Debian);
        // assert
        assert!(steps.is_empty());
    }

    #[test]
    fn install_steps_empty_when_no_install_section() {
        // arrange
        let mut recipe = make_recipe(None, None);
        recipe.install = None;
        // act
        let steps = recipe.install_steps_for(&Platform::MacOS);
        // assert
        assert!(steps.is_empty());
    }

    #[test]
    fn recipe_kind_display_brew() {
        // arrange / act
        let kind = RecipeKind::Brew;
        // assert
        assert_eq!(kind.to_string(), "brew");
    }

    #[test]
    fn recipe_kind_display_qwert() {
        // arrange / act
        let kind = RecipeKind::Qwert;
        // assert
        assert_eq!(kind.to_string(), "qwert");
    }
}
