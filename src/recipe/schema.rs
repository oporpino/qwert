use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Recipe {
    pub meta: RecipeMeta,
    pub check: Option<RecipeCheck>,
    pub install: Option<RecipeInstall>,
    pub upgrade: Option<RecipeUpgrade>,
    pub uninstall: Option<RecipeUninstall>,
    pub setup: Option<RecipeSetup>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RecipeMeta {
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(rename = "type")]
    pub kind: RecipeKind,
    #[serde(default)]
    pub depends: Vec<String>,
    pub pkg: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RecipeKind {
    Brew,
    Apt,
    Pacman,
    Qwert,
}

impl std::fmt::Display for RecipeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecipeKind::Brew => write!(f, "brew"),
            RecipeKind::Apt => write!(f, "apt"),
            RecipeKind::Pacman => write!(f, "pacman"),
            RecipeKind::Qwert => write!(f, "qwert"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RecipeCheck {
    pub command: String,
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
    pub debian: Option<Commands>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RecipeUpgrade {
    pub macos: Option<Commands>,
    pub debian: Option<Commands>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RecipeUninstall {
    pub macos: Option<Commands>,
    pub debian: Option<Commands>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RecipeSetup {
    pub src: Option<String>,
    pub dest: String,
    #[serde(default)]
    pub symlink: bool,
    pub macos: Option<Commands>,
    pub debian: Option<Commands>,
    pub undo: Option<SetupUndo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SetupUndo {
    pub macos: Option<Commands>,
    pub debian: Option<Commands>,
}

/// Parsed from install.toml
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InstallFile {
    pub meta: RecipeMeta,
    pub check: Option<RecipeCheck>,
    pub install: Option<RecipeInstall>,
    pub upgrade: Option<RecipeUpgrade>,
    pub uninstall: Option<RecipeUninstall>,
}

/// Parsed from setup.toml (flat, no sections except [undo])
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SetupFile {
    pub src: Option<String>,
    pub dest: String,
    #[serde(default)]
    pub symlink: bool,
    pub macos: Option<Commands>,
    pub debian: Option<Commands>,
    pub undo: Option<SetupUndo>,
}

fn platform_cmds<'a>(platform: &crate::platform::Platform, macos: Option<&'a Commands>, debian: Option<&'a Commands>) -> Vec<&'a str> {
    let cmds = match platform {
        crate::platform::Platform::MacOS => macos,
        crate::platform::Platform::Debian | crate::platform::Platform::Arch | crate::platform::Platform::Unknown => debian,
    };
    cmds.map(|c| c.as_steps()).unwrap_or_default()
}

impl Recipe {
    pub fn install_steps_for(&self, platform: &crate::platform::Platform) -> Vec<&str> {
        let Some(s) = &self.install else { return vec![] };
        platform_cmds(platform, s.macos.as_ref(), s.debian.as_ref())
    }

    pub fn uninstall_steps_for(&self, platform: &crate::platform::Platform) -> Vec<&str> {
        let Some(s) = &self.uninstall else { return vec![] };
        platform_cmds(platform, s.macos.as_ref(), s.debian.as_ref())
    }

    pub fn upgrade_steps_for(&self, platform: &crate::platform::Platform) -> Vec<&str> {
        let Some(s) = &self.upgrade else { return vec![] };
        platform_cmds(platform, s.macos.as_ref(), s.debian.as_ref())
    }
}

impl RecipeSetup {
    pub fn setup_cmds_for(&self, platform: &crate::platform::Platform) -> Vec<&str> {
        platform_cmds(platform, self.macos.as_ref(), self.debian.as_ref())
    }

    pub fn undo_cmds_for(&self, platform: &crate::platform::Platform) -> Vec<&str> {
        let Some(undo) = &self.undo else { return vec![] };
        platform_cmds(platform, undo.macos.as_ref(), undo.debian.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::Platform;

    fn make_recipe(install_macos: Option<Commands>, install_debian: Option<Commands>) -> Recipe {
        Recipe {
            meta: RecipeMeta {
                name: "test".into(),
                version: "1.0.0".into(),
                description: "test recipe".into(),
                kind: RecipeKind::Brew,
                depends: vec![],
                pkg: None,
            },
            check: None,
            install: Some(RecipeInstall { macos: install_macos, debian: install_debian }),
            upgrade: None,
            uninstall: None,
            setup: None,
        }
    }

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
        // arrange
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
        // arrange
        let kind = RecipeKind::Brew;
        // act
        let result = kind.to_string();
        // assert
        assert_eq!(result, "brew");
    }

    #[test]
    fn recipe_kind_display_qwert() {
        // arrange
        let kind = RecipeKind::Qwert;
        // act
        let result = kind.to_string();
        // assert
        assert_eq!(result, "qwert");
    }

    #[test]
    fn uninstall_steps_empty_when_no_section_and_brew_kind() {
        // arrange
        let recipe = make_recipe(None, None);
        // act
        let steps = recipe.uninstall_steps_for(&Platform::MacOS);
        // assert
        assert!(steps.is_empty());
    }

    #[test]
    fn uninstall_steps_uses_explicit_macos_section() {
        // arrange
        let mut recipe = make_recipe(None, None);
        recipe.uninstall = Some(RecipeUninstall {
            macos: Some(Commands::One("brew uninstall tmux".into())),
            debian: None,
        });
        // act
        let steps = recipe.uninstall_steps_for(&Platform::MacOS);
        // assert
        assert_eq!(steps, vec!["brew uninstall tmux"]);
    }

    #[test]
    fn depends_field_parsed_from_toml() {
        // arrange
        let toml = r#"
[meta]
name = "lvim"
version = "1.0.0"
description = "LunarVim"
type = "qwert"
depends = ["neovim"]

[check]
command = "lvim"
"#;
        // act
        let recipe: InstallFile = toml::from_str(toml).unwrap();
        // assert
        assert_eq!(recipe.meta.depends, vec!["neovim"]);
    }

    #[test]
    fn setup_cmds_for_macos_returns_macos_commands() {
        // arrange
        let setup = RecipeSetup {
            src: None,
            dest: "~/.config/iterm2".into(),
            symlink: false,
            macos: Some(Commands::One("defaults write com.foo bar".into())),
            debian: None,
            undo: None,
        };
        // act
        let cmds = setup.setup_cmds_for(&Platform::MacOS);
        // assert
        assert_eq!(cmds, vec!["defaults write com.foo bar"]);
    }

    #[test]
    fn setup_cmds_for_debian_returns_debian_commands() {
        // arrange
        let setup = RecipeSetup {
            src: None,
            dest: "/etc/foo".into(),
            symlink: false,
            macos: None,
            debian: Some(Commands::One("ln -s foo bar".into())),
            undo: None,
        };
        // act
        let cmds = setup.setup_cmds_for(&Platform::Debian);
        // assert
        assert_eq!(cmds, vec!["ln -s foo bar"]);
    }

    #[test]
    fn setup_cmds_for_returns_empty_when_no_commands() {
        // arrange
        let setup = RecipeSetup {
            src: None,
            dest: "~/.tmux.conf".into(),
            symlink: true,
            macos: None,
            debian: None,
            undo: None,
        };
        // act
        let cmds = setup.setup_cmds_for(&Platform::MacOS);
        // assert
        assert!(cmds.is_empty());
    }

    #[test]
    fn undo_cmds_for_returns_undo_commands() {
        // arrange
        let setup = RecipeSetup {
            src: None,
            dest: "~/.config/iterm2".into(),
            symlink: false,
            macos: None,
            debian: None,
            undo: Some(SetupUndo {
                macos: Some(Commands::One("defaults delete com.foo bar".into())),
                debian: None,
            }),
        };
        // act
        let cmds = setup.undo_cmds_for(&Platform::MacOS);
        // assert
        assert_eq!(cmds, vec!["defaults delete com.foo bar"]);
    }

    #[test]
    fn undo_cmds_for_returns_empty_when_no_undo_section() {
        // arrange
        let setup = RecipeSetup {
            src: None,
            dest: "~/.tmux.conf".into(),
            symlink: true,
            macos: None,
            debian: None,
            undo: None,
        };
        // act
        let cmds = setup.undo_cmds_for(&Platform::MacOS);
        // assert
        assert!(cmds.is_empty());
    }
}
