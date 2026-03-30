use anyhow::Result;
use std::path::PathBuf;

use crate::platform::{InstallerOps, PlatformOps, run_cmd, shared};

pub struct Linux;

impl PlatformOps for Linux {
    fn install(&self, cmd: &str) -> Result<()> {
        run_cmd(cmd)
    }

    fn upgrade(&self, cmd: &str) -> Result<()> {
        run_cmd(cmd)
    }
}

impl InstallerOps for Linux {
    fn binary_path(&self) -> PathBuf {
        PathBuf::from("/opt/qwert/bin/qwert")
    }

    fn symlink_path(&self) -> PathBuf {
        PathBuf::from("/usr/local/bin/qwert")
    }

    fn zsh_completion_path(&self) -> PathBuf {
        PathBuf::from("/usr/share/zsh/vendor-completions/_qwert")
    }

    fn bash_completion_path(&self) -> Option<PathBuf> {
        Some(PathBuf::from("/etc/bash_completion.d/qwert"))
    }

    fn shell_rc_candidates(&self) -> Vec<PathBuf> {
        let home = dirs::home_dir().expect("no home dir");
        vec![home.join(".zshrc"), home.join(".bashrc")]
    }

    fn install_completions(&self) -> Result<()> {
        shared::install_completions_linux(&self.zsh_completion_path(), self.bash_completion_path().as_deref())
    }

    fn configure_shell(&self) -> Result<PathBuf> {
        shared::configure_shell_rc(&self.shell_rc_candidates())
    }
}
