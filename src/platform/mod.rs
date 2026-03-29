use anyhow::Result;

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

    /// Check if a binary exists on PATH
    fn is_installed(&self, binary: &str) -> bool {
        which(binary)
    }

    /// Apply OS-level system preferences (e.g. macOS defaults write)
    fn apply_system_preferences(&self) -> Result<()> {
        Ok(()) // default: no-op
    }

    /// Default shell for the current user
    fn detect_shell(&self) -> Shell {
        std::env::var("SHELL")
            .ok()
            .and_then(|s| {
                if s.contains("zsh") {
                    Some(Shell::Zsh)
                } else if s.contains("bash") {
                    Some(Shell::Bash)
                } else {
                    None
                }
            })
            .unwrap_or(Shell::Bash)
    }
}

#[derive(Debug, Clone)]
pub enum Shell {
    Zsh,
    Bash,
}

impl Shell {
    pub fn rc_file(&self) -> &'static str {
        match self {
            Shell::Zsh => ".zshrc",
            Shell::Bash => ".bashrc",
        }
    }
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
