use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{Shell, generate};

use crate::cli::Cli;

pub fn run(shell: &str) -> Result<()> {
    let shell = shell.parse::<Shell>()
        .map_err(|_| anyhow::anyhow!("unsupported shell: {}. Use: bash, zsh, fish", shell))?;

    generate(shell, &mut Cli::command(), "qwert", &mut std::io::stdout());
    Ok(())
}
