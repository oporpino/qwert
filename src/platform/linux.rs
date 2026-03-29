use anyhow::Result;
use super::{PlatformOps, run_cmd};

pub struct Linux;

impl PlatformOps for Linux {
    fn install(&self, cmd: &str) -> Result<()> {
        run_cmd(cmd)
    }

    fn upgrade(&self, cmd: &str) -> Result<()> {
        run_cmd(cmd)
    }
}
