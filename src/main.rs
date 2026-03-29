mod adapters;
mod cli;
mod commands;
mod config;
mod platform;
mod recipe;
mod ui;

use clap::Parser;
use cli::{Cli, Command, ConfigAction, RecipesAction, SelfAction, UseTarget};

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Use { target } => match target {
            UseTarget::Script { hook, path } => commands::use_cmd::use_script(&hook, &path),
            UseTarget::Tool(args) => {
                let name = args.first().map(|s| s.as_str()).unwrap_or("");
                let no_install = args.contains(&"--no-install".to_string());
                let version = args.get(1).filter(|v| !v.starts_with('-')).map(|s| s.as_str());
                commands::use_cmd::use_tool(name, version, no_install)
            }
        },

        Command::Versions { name } => commands::versions_cmd::run(&name),
        Command::SearchComplete { term } => commands::search_complete_cmd::run(&term),

        Command::Install { name } => commands::install_cmd::run(&name),

        Command::Setup { name } => commands::setup_cmd::run(&name),

        Command::Uninstall { name } => commands::uninstall_cmd::run(&name),

        Command::Drop { name } => commands::drop_cmd::run(&name),

        Command::Apply { tool, dry_run } => commands::apply::run(tool.as_deref(), dry_run),

        Command::Status { tool } => commands::status::run(tool.as_deref()),

        Command::Search { name } => commands::search::run(&name),

        Command::List => commands::list::run(),

        Command::Upgrade { tool, all } => {
            let target = if all { None } else { tool.as_deref() };
            commands::upgrade::run(target)
        }

        Command::Reinstall { name } => commands::reinstall::run(&name),

        Command::Version => {
            println!("qwert {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }

        Command::Completions { shell } => commands::completions::run(&shell),

        Command::Hook { phase } => commands::hook::run(&phase),

        Command::Info { name } => commands::info::run(&name),

        Command::Doctor => commands::doctor::run(),

        Command::Config { action } => match action {
            ConfigAction::Edit => commands::config::edit(),
        },

        Command::SelfManage { action } => match action {
            SelfAction::Upgrade => commands::self_cmd::upgrade(),
            SelfAction::Reinstall => commands::self_cmd::reinstall(),
        },

        Command::Recipes { action } => match action {
            RecipesAction::Update => commands::recipes_cmd::update(),
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
