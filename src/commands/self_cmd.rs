use anyhow::{Context, Result};
use std::process::Command;

use crate::ui::printer;

const REPO: &str = "https://github.com/gporpino/qwert";
const API: &str = "https://api.github.com/repos/gporpino/qwert";

pub fn upgrade() -> Result<()> {
    printer::h1("Upgrading qwert...");
    printer::blank();

    let latest = fetch_latest_version()?;
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

pub fn reinstall() -> Result<()> {
    printer::h1("Reinstalling qwert...");
    printer::blank();

    let version = fetch_latest_version()?;
    printer::info(&format!("version: {}", version));
    printer::blank();

    download_and_install(&version)?;
    printer::ok("qwert", &format!("reinstalled ({})", version));
    printer::blank();
    Ok(())
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn detect_target() -> Result<String> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let target = match (os, arch) {
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        _ => anyhow::bail!("unsupported platform: {os}/{arch}"),
    };

    Ok(target.to_string())
}

fn fetch_latest_version() -> Result<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "curl -fsSL '{API}/releases/latest' | grep '\"tag_name\"' | sed 's/.*\"tag_name\": *\"\\(.*\\)\".*/\\1/'"
        ))
        .output()
        .context("failed to fetch latest version from GitHub")?;

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if version.is_empty() {
        anyhow::bail!("could not determine latest version");
    }
    Ok(version)
}

pub fn installed_version() -> Option<String> {
    let path = dirs::home_dir()?.join(".qwert/version");
    std::fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn download_and_install(version: &str) -> Result<()> {
    let target = detect_target()?;
    let bin_dir = dirs::home_dir()
        .context("no home dir")?
        .join(".qwert/bin");
    let bin_path = bin_dir.join("qwert");

    let url = format!("{REPO}/releases/download/{version}/qwert-{target}");
    let tmp = std::env::temp_dir().join("qwert-self-tmp");

    printer::info(&format!("downloading {}", url));

    let status = Command::new("sh")
        .arg("-c")
        .arg(format!("curl -fsSL '{url}' -o '{}'", tmp.display()))
        .status()
        .context("failed to run curl")?;

    if !status.success() {
        anyhow::bail!("download failed: {}", url);
    }

    std::fs::create_dir_all(&bin_dir)?;
    Command::new("chmod").args(["755", &tmp.to_string_lossy()]).status()?;
    std::fs::copy(&tmp, &bin_path).context("failed to replace binary")?;
    std::fs::remove_file(&tmp).ok();

    // Save installed version
    let version_file = dirs::home_dir()
        .context("no home dir")?
        .join(".qwert/version");
    std::fs::write(&version_file, version)?;

    Ok(())
}
