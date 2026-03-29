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

/// Install a recipe on the current platform
pub fn install(recipe: &Recipe) -> RunResult {
    let platform = platform::detect();

    if is_installed(recipe) {
        let version = installed_version(recipe);
        return RunResult::AlreadyInstalled { version };
    }

    let steps = recipe.install_steps_for(&platform);
    if steps.is_empty() {
        return RunResult::NotSupported;
    }

    let ops = platform::current();
    for step in steps {
        if let Err(e) = ops.install(step) {
            return RunResult::Failed(e.to_string());
        }
    }
    RunResult::Installed
}

/// Upgrade a recipe
pub fn upgrade(recipe: &Recipe) -> RunResult {
    let platform = platform::detect();

    let steps = recipe.upgrade_steps_for(&platform);
    if steps.is_empty() {
        return RunResult::NotSupported;
    }

    let ops = platform::current();
    for step in steps {
        if let Err(e) = ops.upgrade(step) {
            return RunResult::Failed(e.to_string());
        }
    }
    RunResult::Installed
}

/// Install a recipe and print status to terminal
pub fn install_with_output(recipe: &Recipe) -> bool {
    let name = &recipe.meta.name;

    match install(recipe) {
        RunResult::AlreadyInstalled { version } => {
            let msg = version
                .as_deref()
                .map(|v| format!("already installed ({})", v))
                .unwrap_or_else(|| "already installed".to_string());
            printer::ok(name, &msg);
            true
        }
        RunResult::Installed => {
            let version = installed_version(recipe);
            let msg = version
                .map(|v| format!("installed ({})", v))
                .unwrap_or_else(|| "installed".to_string());
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

    if is_installed(recipe) {
        let version = installed_version(recipe);
        let msg = version
            .map(|v| format!("installed ({})", v))
            .unwrap_or_else(|| "installed".to_string());
        printer::ok(name, &msg);
    } else {
        printer::failed(name, "not installed");
    }
}
