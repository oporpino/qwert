use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

const REPO: &str = "https://github.com/oporpino/qwert";
const API: &str = "https://api.github.com/repos/oporpino/qwert";

/// Map the current OS/arch to the release target triple.
pub fn detect_target() -> Result<String> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let target = match (os, arch) {
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", "x86_64")  => "x86_64-apple-darwin",
        ("linux", "x86_64")  => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        _ => anyhow::bail!("unsupported platform: {os}/{arch}"),
    };
    Ok(target.to_string())
}

/// Fetch the latest release tag from GitHub.
pub fn fetch_latest_version() -> Result<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "curl -fsSL '{API}/releases/latest' | grep '\"tag_name\"' | sed 's/.*\"tag_name\": *\"\\(.*\\)\".*/\\1/'"
        ))
        .output()
        .context("failed to fetch latest version from GitHub")?;

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if version.is_empty() {
        anyhow::bail!("could not determine latest version");
    }
    Ok(version)
}

/// Download a release binary for the given version and target to a temp file.
/// Returns the temp file path.
pub fn download_binary(version: &str, target: &str) -> Result<PathBuf> {
    let url = format!("{REPO}/releases/download/{version}/qwert-{target}");
    let tmp = std::env::temp_dir().join("qwert-download-tmp");

    let status = Command::new("sh")
        .arg("-c")
        .arg(format!("curl -fsSL '{url}' -o '{}'", tmp.display()))
        .status()
        .context("failed to run curl")?;

    if !status.success() {
        anyhow::bail!("download failed: {}", url);
    }
    Command::new("chmod").args(["755", &tmp.to_string_lossy()]).status()?;
    Ok(tmp)
}

/// Write a shell completion file to a system path using sudo.
/// Generates the completion text in-process via `crate::commands::completions::generate`.
pub fn write_completion_sudo(dest: &Path, shell: &str) -> Result<()> {
    let text = crate::commands::completions::generate(shell)?;
    let tmp = std::env::temp_dir().join(format!("qwert-completion-{}", shell));
    std::fs::write(&tmp, text)?;

    if let Some(parent) = dest.parent() {
        Command::new("sudo")
            .args(["mkdir", "-p", &parent.to_string_lossy()])
            .status()
            .context("sudo mkdir for completion dir failed")?;
    }
    Command::new("sudo")
        .args(["cp", &tmp.to_string_lossy(), &dest.to_string_lossy()])
        .status()
        .context("sudo cp completion file failed")?;
    std::fs::remove_file(&tmp).ok();
    Ok(())
}

/// Resolve the shell rc file from a list of candidates.
/// Returns the first existing file, or creates and returns the last candidate as fallback.
pub fn resolve_rc(candidates: &[PathBuf]) -> Result<PathBuf> {
    if let Some(rc) = candidates.iter().find(|p| p.exists()) {
        return Ok(rc.clone());
    }
    let fallback = candidates.last().context("no rc candidates provided")?;
    std::fs::write(fallback, "")?;
    Ok(fallback.clone())
}

/// Inject qwert hooks into a shell rc file, stripping any existing qwert block first.
pub fn inject_shell_hooks(rc_path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(rc_path).unwrap_or_default();

    let stripped: String = content
        .lines()
        .filter(|l| !l.contains("# qwert") && !l.contains("qwert hook"))
        .map(|l| format!("{}\n", l))
        .collect();

    let before_block = "# qwert\neval \"$(qwert hook before)\"\n\n";
    let init_line = "\neval \"$(qwert hook init)\" # qwert\n";

    std::fs::write(rc_path, format!("{}{}{}", before_block, stripped.trim_start(), init_line))?;
    Ok(())
}

/// Configure the shell rc: resolve the file, then inject hooks.
/// Returns the rc path used.
pub fn configure_shell_rc(candidates: &[PathBuf]) -> Result<PathBuf> {
    let rc = resolve_rc(candidates)?;
    inject_shell_hooks(&rc)?;
    Ok(rc)
}

/// Install zsh and optionally bash completions on Linux distros.
/// Skips a target if its parent directory does not exist on the system.
pub fn install_completions_linux(zsh: &std::path::Path, bash: Option<&std::path::Path>) -> Result<()> {
    if zsh.parent().map(|p| p.exists()).unwrap_or(false) {
        write_completion_sudo(zsh, "zsh")?;
    }
    if let Some(bash_dest) = bash {
        if bash_dest.parent().map(|p| p.exists()).unwrap_or(false) {
            write_completion_sudo(bash_dest, "bash")?;
        }
    }
    Ok(())
}

/// Create (or update) a symlink at `link` pointing to `target`, using sudo.
pub fn create_symlink_sudo(target: &Path, link: &Path) -> Result<()> {
    Command::new("sudo")
        .args(["ln", "-sf", &target.to_string_lossy(), &link.to_string_lossy()])
        .status()
        .context("failed to create symlink")?;
    Ok(())
}

/// Install a binary to a system path using sudo.
pub fn install_binary_sudo(src: &Path, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        Command::new("sudo")
            .args(["mkdir", "-p", &parent.to_string_lossy()])
            .status()
            .context("sudo mkdir failed")?;
    }
    let status = Command::new("sudo")
        .args(["cp", &src.to_string_lossy(), &dest.to_string_lossy()])
        .status()
        .context("failed to install binary (requires sudo)")?;
    if !status.success() {
        anyhow::bail!("sudo cp failed — permission denied");
    }
    Command::new("sudo")
        .args(["chmod", "755", &dest.to_string_lossy()])
        .status()
        .context("sudo chmod failed")?;
    Ok(())
}
