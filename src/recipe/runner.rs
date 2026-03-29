use std::path::{Path, PathBuf};

use super::schema::{Recipe, RecipeSetup};
use crate::config::qwert_yml;
use crate::platform;
use crate::platform::fs as pfs;
use crate::ui::printer;

pub enum RunResult {
    AlreadyInstalled { version: Option<String> },
    Installed,
    Failed(String),
    NotSupported,
}

/// Check if a recipe is already installed
pub fn is_installed(recipe: &Recipe) -> bool {
    if let Some(check) = &recipe.check {
        platform::which(&check.command)
    } else {
        false
    }
}

/// Get the installed version of a recipe
pub fn installed_version(recipe: &Recipe) -> Option<String> {
    let check = recipe.check.as_ref()?;
    let flag = check.version_flag.as_deref()?;
    platform::version_of(&check.command, flag)
}

/// Package name to pass to the adapter
fn pkg_name(recipe: &Recipe) -> &str {
    recipe.meta.pkg.as_deref().unwrap_or(&recipe.meta.name)
}

fn run_install(cmd: &str) -> Result<(), String> {
    platform::current().install(cmd).map_err(|e| e.to_string())
}

fn run_upgrade(cmd: &str) -> Result<(), String> {
    platform::current().upgrade(cmd).map_err(|e| e.to_string())
}

pub fn version_msg(prefix: &str, version: Option<String>) -> String {
    version.map(|v| format!("{} ({})", prefix, v)).unwrap_or_else(|| prefix.to_string())
}

/// Install a recipe on the current platform, resolving dependencies first.
pub fn install(recipe: &Recipe, recipes_dir: &Path) -> RunResult {
    let platform = platform::detect();

    if is_installed(recipe) {
        let version = installed_version(recipe);
        return RunResult::AlreadyInstalled { version };
    }

    for dep_name in &recipe.meta.depends {
        match super::index::find(dep_name, recipes_dir) {
            Some(dep) => {
                if !is_installed(&dep) {
                    printer::installing(dep_name, &format!("dependency of {}...", recipe.meta.name));
                    let result = install(&dep, recipes_dir);
                    if matches!(result, RunResult::Failed(_) | RunResult::NotSupported) {
                        return RunResult::Failed(format!("dependency '{}' failed to install", dep_name));
                    }
                }
            }
            None => return RunResult::Failed(format!("dependency '{}' not found", dep_name)),
        }
    }

    // Try adapter first
    if let Some(adapter) = crate::adapters::for_kind(&recipe.meta.kind) {
        if adapter.available() {
            let cmd = adapter.install_cmd(pkg_name(recipe));
            return match run_install(&cmd) {
                Ok(_) => RunResult::Installed,
                Err(e) => RunResult::Failed(e),
            };
        }
    }

    // Fall back to explicit commands
    let steps = recipe.install_steps_for(&platform);
    if steps.is_empty() {
        return RunResult::NotSupported;
    }

    for step in steps {
        if let Err(e) = run_install(step) {
            return RunResult::Failed(e);
        }
    }
    RunResult::Installed
}

/// Uninstall a recipe on the current platform
pub fn uninstall(recipe: &Recipe) -> RunResult {
    let platform = platform::detect();

    // Try adapter first
    if let Some(adapter) = crate::adapters::for_kind(&recipe.meta.kind) {
        if adapter.available() {
            let cmd = adapter.uninstall_cmd(pkg_name(recipe));
            return match platform::run_cmd(&cmd) {
                Ok(_) => RunResult::Installed,
                Err(e) => RunResult::Failed(e.to_string()),
            };
        }
    }

    // Fall back to explicit commands
    let steps = recipe.uninstall_steps_for(&platform);
    if steps.is_empty() {
        return RunResult::NotSupported;
    }

    for step in steps {
        if let Err(e) = run_install(step) {
            return RunResult::Failed(e);
        }
    }
    RunResult::Installed
}

/// Uninstall a recipe and print status to terminal
pub fn uninstall_with_output(recipe: &Recipe) -> bool {
    let name = &recipe.meta.name;
    printer::installing(name, "uninstalling...");
    match uninstall(recipe) {
        RunResult::Installed => { printer::ok(name, "uninstalled"); true }
        RunResult::NotSupported => { printer::failed(name, "no uninstall command defined"); false }
        RunResult::Failed(err) => { printer::failed(name, &err); false }
        RunResult::AlreadyInstalled { .. } => unreachable!(),
    }
}

/// Upgrade a recipe
pub fn upgrade(recipe: &Recipe) -> RunResult {
    let platform = platform::detect();

    if let Some(adapter) = crate::adapters::for_kind(&recipe.meta.kind) {
        if adapter.available() {
            let cmd = adapter.upgrade_cmd(pkg_name(recipe));
            return match run_upgrade(&cmd) {
                Ok(_) => RunResult::Installed,
                Err(e) => RunResult::Failed(e),
            };
        }
    }

    let steps = recipe.upgrade_steps_for(&platform);
    if steps.is_empty() {
        return RunResult::NotSupported;
    }

    for step in steps {
        if let Err(e) = run_upgrade(step) {
            return RunResult::Failed(e);
        }
    }
    RunResult::Installed
}

/// Install a recipe and print status to terminal
pub fn install_with_output(recipe: &Recipe, recipes_dir: &Path) -> bool {
    let name = &recipe.meta.name;

    match install(recipe, recipes_dir) {
        RunResult::AlreadyInstalled { version } => {
            printer::ok(name, &version_msg("already installed", version));
            true
        }
        RunResult::Installed => {
            let tag = printer::kind_tag(&recipe.meta.kind.to_string());
            let msg = format!("{}  {}", version_msg("installed", installed_version(recipe)), tag);
            printer::ok(name, &msg);
            true
        }
        RunResult::Failed(err) => {
            printer::failed(name, &err);
            false
        }
        RunResult::NotSupported => {
            printer::failed(name, "not supported on this platform");
            false
        }
    }
}

/// Check and print status of a recipe
pub fn status_with_output(recipe: &Recipe) {
    let name = &recipe.meta.name;
    let tag = printer::kind_tag(&recipe.meta.kind.to_string());

    if is_installed(recipe) {
        let msg = format!("{}  {}", version_msg("installed", installed_version(recipe)), tag);
        printer::ok(name, &msg);
    } else {
        printer::failed(name, &format!("not installed  {}", tag));
    }
}

// --- Setup phase ---

/// Resolve the src path: explicit src (expands ~) or default QWERT_CONFIG_DIR/<name>
fn resolve_src(setup: &RecipeSetup, recipe_name: &str, config_dir: &Path) -> PathBuf {
    match &setup.src {
        Some(src) => PathBuf::from(qwert_yml::expand_tilde(src)),
        None => config_dir.join(recipe_name),
    }
}

/// Run the setup phase for a recipe.
pub fn setup(recipe: &Recipe, config_dir: &Path) -> RunResult {
    let Some(s) = &recipe.setup else {
        return RunResult::NotSupported;
    };

    let platform = platform::detect();
    let dest = PathBuf::from(qwert_yml::expand_tilde(&s.dest));

    // Commands-based setup (iterm2, delta, etc.)
    let cmds = s.setup_cmds_for(&platform);
    if !cmds.is_empty() {
        for cmd in cmds {
            if let Err(e) = platform::run_cmd(cmd) {
                return RunResult::Failed(e.to_string());
            }
        }
        return RunResult::Installed;
    }

    // Symlink
    if s.symlink {
        let src = resolve_src(s, &recipe.meta.name, config_dir);
        if dest.is_symlink() && std::fs::read_link(&dest).ok().as_deref() == Some(src.as_path()) {
            return RunResult::AlreadyInstalled { version: None };
        }
        return match pfs::create_symlink(&src, &dest) {
            Ok(_) => RunResult::Installed,
            Err(e) => RunResult::Failed(e.to_string()),
        };
    }

    // Copy
    if dest.exists() {
        return RunResult::AlreadyInstalled { version: None };
    }
    let src = resolve_src(s, &recipe.meta.name, config_dir);
    if !src.exists() {
        return RunResult::Failed(format!("src not found: {}", src.display()));
    }
    match pfs::copy_file(&src, &dest) {
        Ok(_) => RunResult::Installed,
        Err(e) => RunResult::Failed(e.to_string()),
    }
}

/// Run setup and print status to terminal. Returns true on success.
pub fn setup_with_output(recipe: &Recipe, config_dir: &Path) -> bool {
    let name = &recipe.meta.name;
    match setup(recipe, config_dir) {
        RunResult::NotSupported => true,
        RunResult::AlreadyInstalled { .. } => {
            printer::ok(name, "setup already done");
            true
        }
        RunResult::Installed => {
            printer::ok(name, "setup applied");
            true
        }
        RunResult::Failed(err) => {
            printer::failed(name, &format!("setup failed: {}", err));
            false
        }
    }
}

/// Undo the setup phase for a recipe (used by drop/uninstall).
pub fn undo_setup(recipe: &Recipe, _config_dir: &Path) -> RunResult {
    let Some(s) = &recipe.setup else {
        return RunResult::NotSupported;
    };

    let platform = platform::detect();
    let dest = PathBuf::from(qwert_yml::expand_tilde(&s.dest));

    // Commands-based: run undo commands
    let cmds = s.setup_cmds_for(&platform);
    if !cmds.is_empty() {
        let undo_cmds = s.undo_cmds_for(&platform);
        if undo_cmds.is_empty() {
            return RunResult::Failed(format!(
                "no undo commands defined for {} — undo setup manually",
                recipe.meta.name
            ));
        }
        for cmd in undo_cmds {
            if let Err(e) = platform::run_cmd(cmd) {
                return RunResult::Failed(e.to_string());
            }
        }
        return RunResult::Installed;
    }

    // Symlink: just remove it
    if s.symlink {
        if dest.is_symlink() {
            if let Err(e) = std::fs::remove_file(&dest) {
                return RunResult::Failed(e.to_string());
            }
        }
        return RunResult::Installed;
    }

    // Copy: backup then remove
    if dest.exists() {
        let backup_dir = dirs::home_dir()
            .map(|h| h.join(".qwert").join("backups").join(&recipe.meta.name))
            .unwrap_or_else(|| PathBuf::from("/tmp/qwert-backups").join(&recipe.meta.name));

        let filename = dest.file_name().unwrap_or_default();
        let backup_path = backup_dir.join(filename);

        if let Err(e) = pfs::copy_file(&dest, &backup_path) {
            return RunResult::Failed(format!("backup failed: {}", e));
        }
        if let Err(e) = std::fs::remove_file(&dest) {
            return RunResult::Failed(e.to_string());
        }
        printer::info(&format!("backup saved to {}", backup_path.display()));
    }

    RunResult::Installed
}

/// Undo setup and print status to terminal. Returns true on success.
pub fn undo_setup_with_output(recipe: &Recipe, config_dir: &Path) -> bool {
    let name = &recipe.meta.name;
    match undo_setup(recipe, config_dir) {
        RunResult::NotSupported => true,
        RunResult::AlreadyInstalled { .. } => true,
        RunResult::Installed => {
            printer::ok(name, "setup undone");
            true
        }
        RunResult::Failed(err) => {
            printer::failed(name, &format!("undo setup: {}", err));
            false
        }
    }
}

/// Returns a static label for setup status
pub fn setup_status_label(setup: &RecipeSetup, config_dir: &Path, recipe_name: &str) -> &'static str {
    let platform = platform::detect();
    let dest = PathBuf::from(qwert_yml::expand_tilde(&setup.dest));

    // Commands-based: always show "configured" (no reliable idempotency check)
    if !setup.setup_cmds_for(&platform).is_empty() {
        return "configured";
    }

    if setup.symlink {
        let src = resolve_src(setup, recipe_name, config_dir);
        if dest.is_symlink() && std::fs::read_link(&dest).ok().as_deref() == Some(src.as_path()) {
            return "linked";
        }
        return "not linked";
    }

    if dest.exists() { "copied" } else { "not copied" }
}

/// Check and print install + setup status in one line.
pub fn status_with_setup_output(recipe: &Recipe, config_dir: &Path) {
    let name = &recipe.meta.name;
    let tag = printer::kind_tag_col(&recipe.meta.kind.to_string());

    let install_ok = is_installed(recipe);
    let install_str = version_msg(
        if install_ok { "installed" } else { "not installed" },
        if install_ok { installed_version(recipe) } else { None },
    );

    let setup_str = recipe.setup.as_ref()
        .map(|s| setup_status_label(s, config_dir, name))
        .unwrap_or("—");

    // Fixed-width columns: install (28) | kind (9) | setup
    let msg = format!("{:<28}{}  {}", install_str, tag, setup_str);

    if install_ok {
        printer::ok(name, &msg);
    } else {
        printer::failed(name, &msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe::schema::{RecipeMeta, RecipeKind, RecipeCheck, SetupUndo, Commands};
    use std::fs;

    fn make_recipe_with_setup(setup: Option<RecipeSetup>) -> Recipe {
        Recipe {
            meta: RecipeMeta {
                name: "test".into(),
                version: "1.0.0".into(),
                description: "test".into(),
                kind: RecipeKind::Brew,
                depends: vec![],
                pkg: None,
            },
            check: Some(RecipeCheck { command: "test-nonexistent-binary".into(), version_flag: None }),
            install: None,
            upgrade: None,
            uninstall: None,
            setup,
        }
    }

    fn make_setup(dest: &str, symlink: bool, src: Option<&str>) -> RecipeSetup {
        RecipeSetup {
            src: src.map(|s| s.to_string()),
            dest: dest.to_string(),
            symlink,
            macos: None,
            debian: None,
            undo: None,
        }
    }

    #[test]
    fn resolve_src_uses_explicit_src_when_provided() {
        // arrange
        let setup = make_setup("~/.tmux.conf", true, Some("/tmp/my-tmux.conf"));
        let config_dir = std::path::PathBuf::from("/home/user/.config/qwert");
        // act
        let src = resolve_src(&setup, "tmux", &config_dir);
        // assert
        assert_eq!(src, std::path::PathBuf::from("/tmp/my-tmux.conf"));
    }

    #[test]
    fn resolve_src_defaults_to_config_dir_slash_name() {
        // arrange
        let setup = make_setup("~/.tmux.conf", true, None);
        let config_dir = std::path::PathBuf::from("/home/user/.config/qwert");
        // act
        let src = resolve_src(&setup, "tmux", &config_dir);
        // assert
        assert_eq!(src, std::path::PathBuf::from("/home/user/.config/qwert/tmux"));
    }

    #[test]
    fn setup_returns_not_supported_when_no_setup_section() {
        // arrange
        let recipe = make_recipe_with_setup(None);
        let config_dir = std::path::PathBuf::from("/tmp");
        // act
        let result = setup(&recipe, &config_dir);
        // assert
        assert!(matches!(result, RunResult::NotSupported));
    }

    #[test]
    fn setup_symlink_creates_symlink_at_dest() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_runner_test_symlink");
        fs::create_dir_all(&dir).unwrap();
        let src_dir = dir.join("config");
        fs::create_dir_all(&src_dir).unwrap();
        let src_file = src_dir.join("tmux");
        fs::write(&src_file, "config content").unwrap();
        let dest = dir.join("dest").join(".tmux.conf");

        let s = RecipeSetup {
            src: Some(src_file.to_str().unwrap().to_string()),
            dest: dest.to_str().unwrap().to_string(),
            symlink: true,
            macos: None,
            debian: None,
            undo: None,
        };
        let recipe = make_recipe_with_setup(Some(s));
        // act
        let result = setup(&recipe, &src_dir);
        // assert
        assert!(matches!(result, RunResult::Installed));
        assert!(dest.is_symlink());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn setup_symlink_idempotent_when_already_linked() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_runner_test_idempotent");
        fs::create_dir_all(&dir).unwrap();
        let src = dir.join("src_file");
        let dest = dir.join("dest_link");
        fs::write(&src, "data").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&src, &dest).unwrap();

        let s = RecipeSetup {
            src: Some(src.to_str().unwrap().to_string()),
            dest: dest.to_str().unwrap().to_string(),
            symlink: true,
            macos: None,
            debian: None,
            undo: None,
        };
        let recipe = make_recipe_with_setup(Some(s));
        // act
        let result = setup(&recipe, &dir);
        // assert
        assert!(matches!(result, RunResult::AlreadyInstalled { .. }));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn setup_copy_returns_already_installed_when_dest_exists() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_runner_test_copy_exists");
        fs::create_dir_all(&dir).unwrap();
        let dest = dir.join("dest.conf");
        fs::write(&dest, "existing").unwrap();

        let s = make_setup(dest.to_str().unwrap(), false, None);
        let recipe = make_recipe_with_setup(Some(s));
        // act
        let result = setup(&recipe, &dir);
        // assert
        assert!(matches!(result, RunResult::AlreadyInstalled { .. }));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn setup_copy_fails_when_src_missing() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_runner_test_copy_fail");
        fs::create_dir_all(&dir).unwrap();
        let dest = dir.join("dest.conf");

        // src defaults to config_dir/test, which doesn't exist
        let s = make_setup(dest.to_str().unwrap(), false, None);
        let recipe = make_recipe_with_setup(Some(s));
        // act
        let result = setup(&recipe, &dir);
        // assert — src = dir/test, doesn't exist
        assert!(matches!(result, RunResult::Failed(_)));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn setup_copy_copies_file_to_dest() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_runner_test_copy_ok");
        fs::create_dir_all(&dir).unwrap();
        let src = dir.join("src.conf");
        let dest = dir.join("dest.conf");
        fs::write(&src, "my config").unwrap();

        let s = RecipeSetup {
            src: Some(src.to_str().unwrap().to_string()),
            dest: dest.to_str().unwrap().to_string(),
            symlink: false,
            macos: None,
            debian: None,
            undo: None,
        };
        let recipe = make_recipe_with_setup(Some(s));
        // act
        let result = setup(&recipe, &dir);
        // assert
        assert!(matches!(result, RunResult::Installed));
        assert_eq!(fs::read_to_string(&dest).unwrap(), "my config");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn undo_setup_removes_symlink() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_runner_test_undo_symlink");
        fs::create_dir_all(&dir).unwrap();
        let src = dir.join("src");
        let dest = dir.join("link");
        fs::write(&src, "data").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&src, &dest).unwrap();

        let s = RecipeSetup {
            src: Some(src.to_str().unwrap().to_string()),
            dest: dest.to_str().unwrap().to_string(),
            symlink: true,
            macos: None,
            debian: None,
            undo: None,
        };
        let recipe = make_recipe_with_setup(Some(s));
        // act
        let result = undo_setup(&recipe, &dir);
        // assert
        assert!(matches!(result, RunResult::Installed));
        assert!(!dest.is_symlink());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn undo_setup_backs_up_and_removes_copy() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_runner_test_undo_copy");
        fs::create_dir_all(&dir).unwrap();
        let dest = dir.join("dest.conf");
        fs::write(&dest, "config data").unwrap();

        let s = RecipeSetup {
            src: None,
            dest: dest.to_str().unwrap().to_string(),
            symlink: false,
            macos: None,
            debian: None,
            undo: None,
        };
        let mut recipe = make_recipe_with_setup(Some(s));
        recipe.meta.name = "mytest".into();
        // act
        let result = undo_setup(&recipe, &dir);
        // assert
        assert!(matches!(result, RunResult::Installed));
        assert!(!dest.exists());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn undo_setup_fails_when_commands_type_and_no_undo_defined() {
        // arrange
        let s = RecipeSetup {
            src: None,
            dest: "~/.config/iterm2".into(),
            symlink: false,
            macos: Some(Commands::One("defaults write com.foo bar".into())),
            debian: None,
            undo: None,
        };
        let recipe = make_recipe_with_setup(Some(s));
        // act
        let result = undo_setup(&recipe, std::path::Path::new("/tmp"));
        // assert
        assert!(matches!(result, RunResult::Failed(_)));
    }

    #[test]
    fn setup_status_label_returns_dash_when_no_section() {
        // arrange
        let recipe = make_recipe_with_setup(None);
        let config_dir = std::path::PathBuf::from("/tmp");
        // act
        let label = recipe.setup.as_ref()
            .map(|s| setup_status_label(s, &config_dir, &recipe.meta.name))
            .unwrap_or("—");
        // assert
        assert_eq!(label, "—");
    }

    #[test]
    fn setup_status_label_returns_not_linked_when_no_symlink() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_runner_test_label");
        fs::create_dir_all(&dir).unwrap();
        let dest = dir.join("nonexistent.conf");
        let s = make_setup(dest.to_str().unwrap(), true, None);
        let recipe = make_recipe_with_setup(Some(s));
        // act
        let label = recipe.setup.as_ref()
            .map(|s| setup_status_label(s, &dir, &recipe.meta.name))
            .unwrap_or("—");
        // assert
        assert_eq!(label, "not linked");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn setup_status_label_returns_linked_when_correct_symlink_exists() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_runner_test_label_linked");
        fs::create_dir_all(&dir).unwrap();
        let src = dir.join("test");
        let dest = dir.join("link.conf");
        fs::write(&src, "data").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&src, &dest).unwrap();

        let s = RecipeSetup {
            src: Some(src.to_str().unwrap().to_string()),
            dest: dest.to_str().unwrap().to_string(),
            symlink: true,
            macos: None,
            debian: None,
            undo: None,
        };
        let recipe = make_recipe_with_setup(Some(s));
        // act
        let label = recipe.setup.as_ref()
            .map(|sl| setup_status_label(sl, &dir, &recipe.meta.name))
            .unwrap_or("—");
        // assert
        assert_eq!(label, "linked");
        fs::remove_dir_all(&dir).ok();
    }
}
