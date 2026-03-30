use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "qwert",
    about = "Dev environment manager",
    version,
    propagate_version = true,
    disable_help_subcommand = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Declare a tool, install it, and run setup (full operation)
    Use {
        #[command(subcommand)]
        target: UseTarget,
    },

    /// Declare a tool and install it (skips setup)
    Install {
        /// Tool name
        name: String,
    },

    /// Run setup for a tool (symlinks, config, commands)
    Setup {
        /// Tool name
        name: String,
    },

    /// Remove tool declaration and uninstall it (no setup undo)
    Uninstall {
        /// Tool name
        name: String,
    },

    /// Full teardown: remove declaration, uninstall, and undo setup (with backup)
    Drop {
        /// Tool name
        name: String,
    },

    /// Apply qwert.yml to the machine — install and setup everything declared
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

    /// Search recipes and brew by name
    Search {
        /// Tool name to search for
        name: String,
    },

    /// Generate shell completion script
    Completions {
        /// Shell: bash, zsh, fish
        shell: String,
    },

    /// Print available versions for a tool (used by shell completions)
    #[command(name = "_versions", hide = true)]
    Versions {
        /// Tool name
        name: String,
    },

    /// Search tools and output name\tdescription per line (used by shell completions)
    #[command(name = "_search", hide = true)]
    SearchComplete {
        /// Search term
        term: String,
    },

    /// Show full details for a tool (recipe, install status, setup)
    Info {
        /// Tool name
        name: String,
    },

    /// Output shell hook for before or init phase (eval in .zshrc)
    Hook {
        /// Phase: before or init
        phase: String,
    },

    /// Show help
    Help,

    /// List declared tools and their status
    List,

    /// Upgrade tools
    Upgrade {
        /// Upgrade only this tool
        tool: Option<String>,
        /// Upgrade all declared tools
        #[arg(long)]
        all: bool,
    },

    /// Reinstall a tool
    Reinstall {
        /// Tool name
        name: String,
    },

    /// Show qwert version
    Version,

    /// Health check — verify installation and symlinks
    Doctor,

    /// Open qwert.yml in $EDITOR
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Manage qwert itself
    #[command(name = "self")]
    SelfManage {
        #[command(subcommand)]
        action: SelfAction,
    },

    /// Manage the recipe index
    Recipes {
        #[command(subcommand)]
        action: RecipesAction,
    },
}

#[derive(Subcommand)]
pub enum UseTarget {
    /// Use a tool recipe
    #[command(external_subcommand)]
    Tool(Vec<String>),

    /// Add a script to zsh before or init hooks
    Script {
        /// Hook: before or init
        hook: String,
        /// Path to the script
        #[arg(long)]
        path: String,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Open qwert.yml in $EDITOR
    Edit,
}

#[derive(Subcommand)]
pub enum SelfAction {
    /// Upgrade qwert to the latest version
    Upgrade,
    /// Reinstall qwert
    Reinstall,
}

#[derive(Subcommand)]
pub enum RecipesAction {
    /// Fetch the latest recipe index
    Update,
}
