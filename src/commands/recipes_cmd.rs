use anyhow::Result;

use crate::ui::printer;

pub fn update() -> Result<()> {
    printer::h1("Updating recipes...");
    printer::blank();

    // TODO: fetch latest recipes from GitHub release and unpack to ~/.qwert/recipes/
    printer::info("Recipe update not yet implemented. Re-run the install script to refresh:");
    printer::bullet("sh -c \"$(curl -fsSL https://raw.githubusercontent.com/gporpino/qwert/main/scripts/install.sh)\"");

    printer::blank();
    Ok(())
}

/// Silent best-effort update — used automatically before use/install/setup.
/// Errors are ignored so offline usage is unaffected.
pub fn update_silent() {
    // TODO: implement
}
