use anyhow::Result;
use std::path::PathBuf;

pub mod fs;
pub mod impls;
pub mod shared;

/// Runtime data directory for qwert: ~/.local/share/qwert/
pub fn data_dir() -> PathBuf {
    dirs::home_dir()
        .expect("no home dir")
        .join(".local/share/qwert")
}

/// Platform-specific installation conventions (paths, completions, shell config).
pub trait InstallerOps {
    /// /opt/qwert/bin/qwert
    fn binary_path(&self) -> PathBuf;

    /// /usr/local/bin/qwert
    fn symlink_path(&self) -> PathBuf;

    /// System zsh completion path (e.g. /usr/local/share/zsh/site-functions/_qwert)
    fn zsh_completion_path(&self) -> PathBuf;

    /// System bash completion path — None on platforms where bash completions are not standard
    fn bash_completion_path(&self) -> Option<PathBuf>;

    /// Shell rc file candidates in priority order (first existing file wins)
    fn shell_rc_candidates(&self) -> Vec<PathBuf>;

    /// Install shell completions to system paths (requires sudo)
    fn install_completions(&self) -> Result<()>;

    /// Inject qwert hooks into the user's shell rc. Returns the rc file path used.
    fn configure_shell(&self) -> Result<PathBuf>;
}

/// Returns the platform-specific installer implementation.
pub fn installer() -> Box<dyn InstallerOps> {
    match detect() {
        Platform::MacOS => Box::new(impls::macos::MacOS),
        Platform::Debian => Box::new(impls::debian::Debian),
        Platform::Arch => Box::new(impls::arch::Arch),
        Platform::Unknown => Box::new(impls::linux::Linux),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    MacOS,
    /// Debian-based Linux (Ubuntu, Debian, etc.) — uses apt-get
    Debian,
    /// Arch-based Linux (Arch, Manjaro, etc.) — uses pacman
    Arch,
    Unknown,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::MacOS => write!(f, "macOS"),
            Platform::Debian => write!(f, "Debian Linux"),
            Platform::Arch => write!(f, "Arch Linux"),
            Platform::Unknown => write!(f, "unknown"),
        }
    }
}

pub fn detect() -> Platform {
    if cfg!(target_os = "macos") {
        return Platform::MacOS;
    }
    if cfg!(target_os = "linux") {
        if std::path::Path::new("/usr/bin/apt-get").exists() {
            return Platform::Debian;
        }
        if std::path::Path::new("/usr/bin/pacman").exists() {
            return Platform::Arch;
        }
    }
    Platform::Unknown
}

/// Core operations that vary by platform.
/// Implement this trait per platform — override only what differs.
pub trait PlatformOps {
    /// Run a shell install command (e.g. "brew install neovim")
    fn install(&self, cmd: &str) -> Result<()>;

    /// Run a shell upgrade command
    fn upgrade(&self, cmd: &str) -> Result<()>;
}

/// Execute a shell command, streaming stdout/stderr to terminal
pub fn run_cmd(cmd: &str) -> Result<()> {
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("command failed: {}", cmd)
    }
}

/// Execute a shell command, capturing stderr; on failure returns stderr content
pub fn run_cmd_capture(cmd: &str) -> Result<(), String> {
    let out = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|e| e.to_string())?;

    if out.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(if stderr.is_empty() {
            format!("command failed: {}", cmd)
        } else {
            stderr
        })
    }
}

/// Check if a binary exists on PATH
pub fn which(binary: &str) -> bool {
    std::process::Command::new("which")
        .arg("-s")
        .arg(binary)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Get the installed version of a binary
pub fn version_of(binary: &str, flag: &str) -> Option<String> {
    std::process::Command::new(binary)
        .arg(flag)
        .output()
        .ok()
        .and_then(|out| {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let combined = format!("{}{}", stdout, stderr);
            combined.lines().next().map(|l| l.trim().to_string())
        })
}

/// Get the current platform ops implementation
pub fn current() -> Box<dyn PlatformOps> {
    match detect() {
        Platform::MacOS => Box::new(impls::macos::MacOS),
        Platform::Debian => Box::new(impls::debian::Debian),
        Platform::Arch => Box::new(impls::arch::Arch),
        Platform::Unknown => Box::new(impls::linux::Linux),
    }
}
