use anyhow::Result;
use super::{PlatformOps, run_cmd};

pub struct MacOS;

impl PlatformOps for MacOS {
    fn install(&self, cmd: &str) -> Result<()> {
        run_cmd(cmd)
    }

    fn upgrade(&self, cmd: &str) -> Result<()> {
        run_cmd(cmd)
    }

    fn apply_system_preferences(&self) -> Result<()> {
        // macOS-specific system defaults can be applied here
        // e.g. disable Ctrl+Space input source switcher (conflicts with tmux prefix)
        Ok(())
    }
}
