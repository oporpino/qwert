use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{Shell, generate};

use crate::cli::Cli;

// Zsh completion is hand-written for dynamic version/tool lookups.
// Bash and fish fall back to clap_complete.
const ZSH_COMPLETION: &str = include_str!("../completions/_qwert");

pub fn run(shell: &str) -> Result<()> {
    if shell == "zsh" {
        print!("{}", ZSH_COMPLETION);
        return Ok(());
    }

    let shell = shell.parse::<Shell>()
        .map_err(|_| anyhow::anyhow!("unsupported shell: {}. Use: bash, zsh, fish", shell))?;

    generate(shell, &mut Cli::command(), "qwert", &mut std::io::stdout());
    Ok(())
}
