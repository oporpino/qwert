use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{Shell, generate as clap_generate};

use crate::cli::Cli;

// Zsh completion is hand-written for dynamic version/tool lookups.
// Bash and fish fall back to clap_complete.
const ZSH_COMPLETION: &str = include_str!("../completions/_qwert");

/// Generate completion text for the given shell as a String.
pub fn generate(shell: &str) -> Result<String> {
    if shell == "zsh" {
        return Ok(ZSH_COMPLETION.to_string());
    }

    let sh = shell.parse::<Shell>()
        .map_err(|_| anyhow::anyhow!("unsupported shell: {}. Use: bash, zsh, fish", shell))?;

    let mut buf = Vec::new();
    clap_generate(sh, &mut Cli::command(), "qwert", &mut buf);
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

pub fn run(shell: &str) -> Result<()> {
    print!("{}", generate(shell)?);
    Ok(())
}
