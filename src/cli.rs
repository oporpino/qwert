use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "qwert",
    about = "Dev environment manager",
    version,
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Declare that this machine uses a tool (adds to qwert.yml and installs)
    Use {
        #[command(subcommand)]
        target: UseTarget,
    },

    /// Remove a tool declaration from this machine
    Drop {
        /// Tool name
        name: String,
        /// Also uninstall the tool
        #[arg(long)]
        uninstall: bool,
    },

    /// Apply qwert.yml to the machine — install everything declared
    Apply {
        /// Apply only this tool
        tool: Option<String>,
        /// Show what would be done without executing
        #[arg(long)]
        dry_run: bool,
    },

    /// Show status of all declared tools
    Status {
        /// Check only this tool
        tool: Option<String>,
    },

    /// Search available recipes
    Search {
        /// Search term (optional, lists all if omitted)
        term: Option<String>,
    },

    /// List declared tools and their status
    List,

    /// Upgrade tools
    Upgrade {
        /// Upgrade only this tool
        tool: Option<String>,
    },

    /// Reinstall a tool
    Reinstall {
        /// Tool name
        name: String,
    },

    /// Update qwert itself and refresh recipe index
    Update,

    /// Show qwert version
    Version,

    /// Health check — verify installation, configs, and symlinks
    Doctor,

    /// Open qwert.yml in $EDITOR
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
pub enum UseTarget {
    /// Use a tool recipe
    #[command(external_subcommand)]
    Tool(Vec<String>),

    /// Add a script to zsh init or end hooks
    Script {
        /// Hook: init or end
        hook: String,
        /// Path to the script
        #[arg(long)]
        path: String,
        /// Only add to qwert.yml, don't install
        #[arg(long)]
        no_install: bool,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Open qwert.yml in $EDITOR
    Edit,
}
