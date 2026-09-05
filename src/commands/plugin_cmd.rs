use anyhow::Result;

use crate::plugins;
use crate::ui::printer;

pub fn add(url: &str) -> Result<()> {
    plugins::add(url)
}

pub fn remove(name: &str) -> Result<()> {
    plugins::remove(name)
}

pub fn list() -> Result<()> {
    let plugins = plugins::list()?;
    printer::h1("Plugins");
    printer::blank();
    if plugins.is_empty() {
        printer::info("No plugins declared. Run `qwert plugin add <url>` to add one.");
        printer::blank();
        return Ok(());
    }
    for p in &plugins {
        let status = if p.cloned { "cloned" } else { "not cloned" };
        printer::field(&p.name, &format!("{}  ({})", p.url, status));
    }
    printer::blank();
    Ok(())
}

pub fn update() -> Result<()> {
    printer::h1("Updating plugins...");
    printer::blank();
    plugins::update_all()
}