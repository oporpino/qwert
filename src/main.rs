mod adapters;
mod cli;
mod commands;
mod config;
mod platform;
mod recipe;
mod ui;

use clap::Parser;
use cli::{Cli, Command, ConfigAction, UseTarget};

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Use { target } => match target {
            UseTarget::Script { hook, path, .. } => commands::use_cmd::use_script(&hook, &path),
            UseTarget::Tool(args) => {
                let name = args.first().map(|s| s.as_str()).unwrap_or("");
                let no_install = args.contains(&"--no-install".to_string());
                commands::use_cmd::use_tool(name, no_install)
            }
        },

        Command::Drop { name, uninstall } => commands::drop_cmd::run(&name, uninstall),

        Command::Apply { tool, dry_run } => commands::apply::run(tool.as_deref(), dry_run),

        Command::Status { tool } => commands::status::run(tool.as_deref()),

        Command::Search { name } => commands::search::run(&name),

        Command::List => commands::list::run(),

        Command::Upgrade { tool } => commands::upgrade::run(tool.as_deref()),

        Command::Reinstall { name } => commands::reinstall::run(&name),

        Command::Update => commands::update::run(),

        Command::Version => {
            println!("qwert {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }

        Command::Doctor => commands::doctor::run(),

        Command::Config { action } => match action {
            ConfigAction::Edit => commands::config::edit(),
        },

        Command::Help => {
            commands::help::run();
            Ok(())
        }
    };

    if let Err(e) = result {
        ui::printer::error(&e.to_string());
        std::process::exit(1);
    }
}
