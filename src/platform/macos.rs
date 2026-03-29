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
}
