use anyhow::{Context, Result};

use super::PackageAdapter;

pub struct BrewAdapter;

impl PackageAdapter for BrewAdapter {
    fn available(&self) -> bool { crate::platform::which("brew") }
    fn install_cmd(&self, pkg: &str) -> String { format!("brew install {}", pkg) }
    fn upgrade_cmd(&self, pkg: &str) -> String { format!("brew upgrade {}", pkg) }
    fn uninstall_cmd(&self, pkg: &str) -> String { format!("brew uninstall {}", pkg) }

    fn ensure(&self) -> Result<()> {
        if self.available() {
            return Ok(());
        }
        install_homebrew()?;
        load_brew_env()?;
        if !self.available() {
            anyhow::bail!("brew installed but still not found in PATH — open a new shell and retry");
        }
        Ok(())
    }
}

/// Download and run the official Homebrew installer, then add shellenv to the shell rc.
fn install_homebrew() -> Result<()> {
    crate::platform::run_cmd(
        r#"NONINTERACTIVE=1 /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)""#,
    )
    .context("failed to install Homebrew")?;

    // Inject `eval "$(brew shellenv)"` into the shell rc for future sessions
    configure_brew_shellenv();
    Ok(())
}

/// Append `eval "$(brew shellenv)"` to the user's shell rc if not already present.
fn configure_brew_shellenv() {
    let brew_bin = brew_binary_path();
    let snippet = format!(r#"eval "$({} shellenv)" # homebrew"#, brew_bin);

    let rc = dirs::home_dir().map(|h| h.join(".zshrc")).unwrap_or_default();
    if !rc.exists() {
        return;
    }
    let content = std::fs::read_to_string(&rc).unwrap_or_default();
    if content.contains("brew shellenv") {
        return;
    }
    let _ = std::fs::write(&rc, format!("{}\n{}\n", content.trim_end(), snippet));
}

/// Source `brew shellenv` into the current process environment so subsequent
/// `brew install` calls work without requiring a new shell session.
fn load_brew_env() -> Result<()> {
    let brew_bin = brew_binary_path();
    if !std::path::Path::new(&brew_bin).exists() {
        return Ok(());
    }

    let out = std::process::Command::new(&brew_bin)
        .arg("shellenv")
        .output()
        .context("failed to run brew shellenv")?;

    for line in String::from_utf8_lossy(&out.stdout).lines() {
        // Parse `export KEY=VALUE` lines
        if let Some(rest) = line.strip_prefix("export ") {
            if let Some((key, val)) = rest.split_once('=') {
                let val = val.trim_matches('"');
                if key == "PATH" {
                    // Prepend brew paths to the current PATH
                    let current = std::env::var("PATH").unwrap_or_default();
                    let new_path = format!("{}:{}", val.trim_end_matches(':'), current);
                    std::env::set_var("PATH", new_path);
                } else {
                    std::env::set_var(key, val);
                }
            }
        }
    }
    Ok(())
}

/// Returns the expected brew binary path based on architecture.
fn brew_binary_path() -> String {
    if std::env::consts::ARCH == "aarch64" {
        "/opt/homebrew/bin/brew".to_string()
    } else {
        "/usr/local/bin/brew".to_string()
    }
}

#[cfg(test)]
#[path = "tests/brew.rs"]
mod tests;
