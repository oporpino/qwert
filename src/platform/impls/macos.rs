use anyhow::Result;
use std::path::PathBuf;

use crate::platform::{InstallerOps, PlatformOps, run_cmd, shared};

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
        PathBuf::from("/usr/local/share/zsh/site-functions/_qwert")
    }

    fn bash_completion_path(&self) -> Option<PathBuf> {
        None
    }

    fn shell_rc_candidates(&self) -> Vec<PathBuf> {
        vec![dirs::home_dir().expect("no home dir").join(".zshrc")]
    }

    fn install_completions(&self) -> Result<()> {
        shared::write_completion_sudo(&self.zsh_completion_path(), "zsh")
    }

    fn configure_shell(&self) -> Result<PathBuf> {
        shared::configure_shell_rc(&self.shell_rc_candidates())
    }
}

