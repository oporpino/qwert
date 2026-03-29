use anyhow::Result;

use crate::ui::printer;

pub fn upgrade() -> Result<()> {
    printer::h1("Upgrading qwert...");
    printer::blank();

    // TODO: fetch latest release from GitHub and replace binary
    printer::info("Self-upgrade not yet implemented. Re-run the install script:");
    printer::bullet("sh -c \"$(curl -fsSL https://raw.githubusercontent.com/gporpino/qwert/main/scripts/install.sh)\"");

    printer::blank();
    Ok(())
}

pub fn reinstall() -> Result<()> {
    printer::h1("Reinstalling qwert...");
    printer::blank();

    // TODO: remove current binary and re-run install script
    printer::info("Self-reinstall not yet implemented. Re-run the install script:");
    printer::bullet("sh -c \"$(curl -fsSL https://raw.githubusercontent.com/gporpino/qwert/main/scripts/install.sh)\"");

    printer::blank();
    Ok(())
}
