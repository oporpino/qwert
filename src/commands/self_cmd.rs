use anyhow::{Context, Result};

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

/// If this process runs under `sudo` (older installers wrapped `self install` in sudo),
/// the HOME env points at the privileged user (e.g. /root). Return the invoking user's
/// real home so user-scoped steps (recipes cache, shell rc, machine identity) land in
/// the right place instead of the root user's.
fn invoking_user_home() -> Option<String> {
    let sudo_user = std::env::var("SUDO_USER").ok()?;
    if sudo_user.is_empty() || sudo_user == "root" {
        return None;
    }
    let current = std::env::var("HOME").unwrap_or_default();
    let out = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("echo ~{}", sudo_user))
        .output()
        .ok()?;
    let home = String::from_utf8_lossy(&out.stdout).trim().to_string();
    // `echo ~user` yields "~user" verbatim when there is no passwd entry.
    if home.is_empty() || home == format!("~{}", sudo_user) || home == current {
        None
    } else {
        Some(home)
    }
}

pub fn install() -> Result<()> {
    printer::h1("Setting up qwert...");
    printer::blank();

    // Re-home user-scoped steps if an outer sudo set HOME to the privileged user.
    let orig_home = std::env::var("HOME").ok();
    let rehome = invoking_user_home();
    if let Some(h) = &rehome {
        std::env::set_var("HOME", h);
    }

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

    if matches!(platform::detect(), platform::Platform::MacOS) && !platform::which("brew") {
        printer::installing("brew", "installing Homebrew...");
        platform::run_cmd(r#"/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)""#)?;
        printer::ok("brew", "installed");
    }

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
    let version = format!("v{}", env!("CARGO_PKG_VERSION"));
    std::fs::write(data_dir.join("version"), &version)?;
    // Chown after write so the version file itself is also transferred to the user
    if let Ok(user) = std::env::var("SUDO_USER").or_else(|_| std::env::var("USER")) {
        let _ = std::process::Command::new("chown")
            .args(["-R", &user, &data_dir.to_string_lossy()])
            .status();
    }
    printer::ok("version", &version);

    printer::blank();
    printer::info(&format!("restart your shell or run: source {}", rc.display()));
    printer::blank();
    printer::info("tip: version control ~/.qwert in a git repo to replicate your environment on any machine.");
    printer::blank();

    // Machine identity — interactive profile selection when not configured yet
    if !crate::config::machine::machine_path().exists() {
        prompt_profile()?;
    }

    // Restore the original HOME so child steps after install() never inherit the override.
    if rehome.is_some() {
        match orig_home {
            Some(original) => std::env::set_var("HOME", original),
            None => std::env::remove_var("HOME"),
        }
    }

    Ok(())
}

/// Ask which profile this machine should apply (from profiles declared in qwert.yml).
fn prompt_profile() -> Result<()> {
    use std::io::Write;

    let config = crate::config::qwert_yml::QwertConfig::load(&crate::config::qwert_yml::manifest_path())?;
    let available = config.profiles_with_tools();

    printer::h1("Machine profile");
    printer::blank();
    if available.is_empty() {
        printer::info("No profiles declared in config.yml yet. Set them anytime with `qwert profile <name>`.");
        printer::blank();
        return Ok(());
    }

    printer::info(&format!("Available profiles: {}", available.join(", ")));
    print!("  Select a profile or ENTER to skip: ");
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    let profile = line.trim().to_string();

    if !profile.is_empty() {
        let mut machine = crate::config::machine::MachineIdentity::default();
        machine.set_profile(profile);
        machine.save()?;
        printer::ok("profile", "saved — run `qwert apply` to apply this machine's setup");
    } else {
        printer::info("No profile configured. Run `qwert profile <name>` anytime.");
    }
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

    printer::info(&format!("downloading {}", version));
    let tmp = shared::download_binary(version, &target)?;
    shared::install_binary_sudo(&tmp, &installer.binary_path())?;
    std::fs::remove_file(&tmp).ok();
    shared::create_symlink_sudo(&installer.binary_path(), &installer.symlink_path())?;

    let data_dir = platform::data_dir();
    std::fs::create_dir_all(&data_dir)?;
    // Fix ownership if version file was left root-owned by a previous install
    if let Ok(user) = std::env::var("USER") {
        let _ = std::process::Command::new("sudo")
            .args(["chown", "-R", &user, &data_dir.to_string_lossy()])
            .status();
    }
    std::fs::write(data_dir.join("version"), version).context("failed to write version file")?;
    Ok(())
}
