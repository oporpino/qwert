use anyhow::Result;

use crate::ui::printer;

pub fn run() -> Result<()> {
    printer::h1("Updating qwert...");
    printer::blank();

    // TODO: implement version fetch from GitHub API and reinstall
    // This will call the install.sh script or a built-in updater
    printer::info("Update not yet implemented. Run the install script to upgrade:");
    printer::bullet("sh -c \"$(curl -fsSL https://raw.githubusercontent.com/oporpino/qwert/main/scripts/install.sh)\"");

    printer::blank();
    Ok(())
}
