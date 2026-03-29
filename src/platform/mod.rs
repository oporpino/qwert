use anyhow::Result;

pub mod fs;
pub mod macos;
pub mod linux;

#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    MacOS,
    /// Debian-based Linux (Ubuntu, Debian, etc.) — uses apt-get
    Debian,
    Unknown,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::MacOS => write!(f, "macOS"),
            Platform::Debian => write!(f, "Debian Linux"),
            Platform::Unknown => write!(f, "unknown"),
        }
    }
}

pub fn detect() -> Platform {
    if cfg!(target_os = "macos") {
        return Platform::MacOS;
    }
    if cfg!(target_os = "linux") {
        // Detect distro family by checking for apt-get
        if std::path::Path::new("/usr/bin/apt-get").exists() {
            return Platform::Debian;
        }
        // Future: detect dnf/pacman for RedHat/Arch support
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
        Platform::MacOS => Box::new(macos::MacOS),
        Platform::Debian | Platform::Unknown => Box::new(linux::Linux),
    }
}
