use anyhow::Result;

use crate::config::qwert_yml;
use crate::ui::printer;

pub fn edit() -> Result<()> {
    let manifest_path = qwert_yml::manifest_path();

    // Create the file if it doesn't exist yet
    if !manifest_path.exists() {
        let config = qwert_yml::QwertConfig::default();
        config.save(&manifest_path)?;
    }

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

    let status = std::process::Command::new(&editor)
        .arg(&manifest_path)
        .status()?;

    if !status.success() {
        printer::error(&format!("editor '{}' exited with error", editor));
    }

    Ok(())
}
