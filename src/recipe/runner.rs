use super::schema::Recipe;
use crate::platform;
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

/// Package name to pass to the adapter: uses `meta.pkg` override if set, else `meta.name`.
fn pkg_name(recipe: &Recipe) -> &str {
    recipe.meta.pkg.as_deref().unwrap_or(&recipe.meta.name)
}

fn run_install(cmd: &str) -> Result<(), String> {
    platform::current().install(cmd).map_err(|e| e.to_string())
}

fn run_upgrade(cmd: &str) -> Result<(), String> {
    platform::current().upgrade(cmd).map_err(|e| e.to_string())
}

fn version_msg(prefix: &str, version: Option<String>) -> String {
    version.map(|v| format!("{} ({})", prefix, v)).unwrap_or_else(|| prefix.to_string())
}

/// Install a recipe on the current platform, resolving dependencies first.
pub fn install(recipe: &Recipe, recipes_dir: &std::path::Path) -> RunResult {
    let platform = platform::detect();

    if is_installed(recipe) {
        let version = installed_version(recipe);
        return RunResult::AlreadyInstalled { version };
    }

    // Resolve and install dependencies first
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
            return match run_install(&cmd) {
                Ok(_) => RunResult::Installed,
                Err(e) => RunResult::Failed(e),
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

    // Try adapter first
    if let Some(adapter) = crate::adapters::for_kind(&recipe.meta.kind) {
        if adapter.available() {
            let cmd = adapter.upgrade_cmd(pkg_name(recipe));
            return match run_upgrade(&cmd) {
                Ok(_) => RunResult::Installed,
                Err(e) => RunResult::Failed(e),
            };
        }
    }

    // Fall back to explicit commands
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
pub fn install_with_output(recipe: &Recipe, recipes_dir: &std::path::Path) -> bool {
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
