use anyhow::Result;

use crate::platform::{self, shared};
use crate::ui::printer;

pub fn upgrade() -> Result<()> {
    printer::h1("Upgrading qwert...");
    printer::blank();

    let latest = shared::fetch_latest_version()?;
    let current = installed_version();

    if current.as_deref() == Some(latest.as_str()) {
        printer::info(&format!("already on latest version ({})", latest));
        printer::blank();
        return Ok(());
    }

    if let Some(ref cur) = current {
        printer::info(&format!("current: {}", cur));
    }
    printer::info(&format!("latest:  {}", latest));
    printer::blank();

    download_and_install(&latest)?;
    printer::ok("qwert", &format!("upgraded to {}", latest));
    printer::blank();
    Ok(())
}

pub fn install() -> Result<()> {
    printer::h1("Setting up qwert...");
    printer::blank();

    let installer = platform::installer();
    let bin_path = installer.binary_path();

    // Create user dir (~/.qwert/) if it doesn't exist
    let user_dir = crate::config::qwert_yml::config_dir();
    std::fs::create_dir_all(&user_dir)?;

    anyhow::ensure!(
        bin_path.exists(),
        "binary not found at {} — place the binary there first",
        bin_path.display()
    );

    let symlink = installer.symlink_path();
    shared::create_symlink_sudo(&bin_path, &symlink)?;
    printer::ok("symlink", &symlink.to_string_lossy());

    installer.install_completions()?;
    printer::ok("completions", "installed");

    crate::commands::recipes_cmd::update()?;
    printer::ok("recipes", "downloaded");

    let rc = installer.configure_shell()?;
    printer::ok("shell", &rc.to_string_lossy());

    let data_dir = platform::data_dir();
    std::fs::create_dir_all(&data_dir)?;
    std::fs::write(data_dir.join("version"), env!("CARGO_PKG_VERSION"))?;
    printer::ok("version", env!("CARGO_PKG_VERSION"));

    printer::blank();
    printer::info(&format!("restart your shell or run: source {}", rc.display()));
    printer::blank();
    printer::info("tip: version control ~/.qwert in a git repo to replicate your environment on any machine.");
    printer::blank();
    Ok(())
}

pub fn reinstall() -> Result<()> {
    printer::h1("Reinstalling qwert...");
    printer::blank();

    let version = installed_version()
        .ok_or_else(|| anyhow::anyhow!("could not determine installed version — run 'qwert self upgrade' instead"))?;
    printer::info(&format!("version: {}", version));
    printer::blank();

    download_and_install(&version)?;
    printer::ok("qwert", &format!("reinstalled ({})", version));
    printer::blank();
    Ok(())
}

pub fn installed_version() -> Option<String> {
    let path = platform::data_dir().join("version");
    std::fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn download_and_install(version: &str) -> Result<()> {
    let target = shared::detect_target()?;
    let installer = platform::installer();

    printer::info(&format!("downloading v{}", version));
    let tmp = shared::download_binary(version, &target)?;
    shared::install_binary_sudo(&tmp, &installer.binary_path())?;
    std::fs::remove_file(&tmp).ok();
    shared::create_symlink_sudo(&installer.binary_path(), &installer.symlink_path())?;

    let data_dir = platform::data_dir();
    std::fs::create_dir_all(&data_dir)?;
    std::fs::write(data_dir.join("version"), version)?;
    Ok(())
}
