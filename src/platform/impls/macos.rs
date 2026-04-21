use anyhow::Result;
use std::path::PathBuf;

use crate::platform::{InstallerOps, PlatformOps, run_cmd, shared};

fn ensure_brew_shellenv(rc: &std::path::Path) {
    let brew_bin = if std::env::consts::ARCH == "aarch64" {
        "/opt/homebrew/bin/brew"
    } else {
        "/usr/local/bin/brew"
    };
    let snippet = format!(r#"eval "$({} shellenv)" # homebrew"#, brew_bin);
    let content = std::fs::read_to_string(rc).unwrap_or_default();
    if !content.contains("brew shellenv") {
        let _ = std::fs::write(rc, format!("{}\n{}\n", content.trim_end(), snippet));
    }
}

fn brew_prefix() -> Option<PathBuf> {
    std::process::Command::new("brew")
        .arg("--prefix")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| PathBuf::from(String::from_utf8_lossy(&o.stdout).trim().to_string()))
}

pub struct MacOS;

impl PlatformOps for MacOS {
    fn install(&self, cmd: &str) -> Result<()> {
        run_cmd(cmd)
    }

    fn upgrade(&self, cmd: &str) -> Result<()> {
        run_cmd(cmd)
    }
}

impl InstallerOps for MacOS {
    fn binary_path(&self) -> PathBuf {
        PathBuf::from("/opt/qwert/bin/qwert")
    }

    fn symlink_path(&self) -> PathBuf {
        PathBuf::from("/usr/local/bin/qwert")
    }

    fn zsh_completion_path(&self) -> PathBuf {
        brew_prefix()
            .map(|p| p.join("share/zsh/site-functions/_qwert"))
            .unwrap_or_else(|| PathBuf::from("/usr/local/share/zsh/site-functions/_qwert"))
    }

    fn bash_completion_path(&self) -> Option<PathBuf> {
        brew_prefix().map(|p| p.join("etc/bash_completion.d/qwert"))
    }

    fn shell_rc_candidates(&self) -> Vec<PathBuf> {
        vec![dirs::home_dir().expect("no home dir").join(".zshrc")]
    }

    fn install_completions(&self) -> Result<()> {
        shared::install_completions_linux(&self.zsh_completion_path(), self.bash_completion_path().as_deref())
    }

    fn configure_shell(&self) -> Result<PathBuf> {
        let rc = shared::configure_shell_rc(&self.shell_rc_candidates())?;
        ensure_brew_shellenv(&rc);
        Ok(rc)
    }
}

