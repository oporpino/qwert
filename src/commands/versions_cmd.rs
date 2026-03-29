use anyhow::Result;

use crate::recipe::{index, schema::RecipeKind};

/// Print available versions for a tool (used by shell completions).
/// Queries the platform package manager. Outputs one version per line.
pub fn run(name: &str) -> Result<()> {
    let recipes_dir = index::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;

    let kind = if let Some(recipe) = index::find(name, &recipes_dir) {
        recipe.meta.kind
    } else {
        // No recipe — infer from platform
        match crate::platform::detect() {
            crate::platform::Platform::MacOS => RecipeKind::Brew,
            crate::platform::Platform::Debian => RecipeKind::Apt,
            crate::platform::Platform::Unknown => return Ok(()),
        }
    };

    let versions = match kind {
        RecipeKind::Brew => fetch_brew_versions(name),
        RecipeKind::Apt => fetch_apt_versions(name),
        _ => vec![],
    };

    for v in versions {
        println!("{}", v);
    }

    Ok(())
}

fn fetch_brew_versions(name: &str) -> Vec<String> {
    let out = std::process::Command::new("brew")
        .args(["info", "--json=v2", name])
        .output();

    let out = match out {
        Ok(o) if o.status.success() => o,
        _ => return vec![],
    };

    // Parse "stable" version from JSON without pulling in serde_json
    // JSON looks like: {"formulae":[{"versions":{"stable":"3.6a",...},...}]}
    let text = String::from_utf8_lossy(&out.stdout);
    let mut versions = vec![];

    for line in text.lines() {
        let line = line.trim();
        // Match: "stable": "3.6a",
        if let Some(rest) = line.strip_prefix("\"stable\": \"") {
            if let Some(ver) = rest.strip_suffix("\",").or_else(|| rest.strip_suffix('"')) {
                let ver = ver.trim();
                if !ver.is_empty() && ver != "null" {
                    versions.push(ver.to_string());
                    break; // first match is the formula version
                }
            }
        }
    }

    versions
}

fn fetch_apt_versions(name: &str) -> Vec<String> {
    let out = std::process::Command::new("apt-cache")
        .args(["show", name])
        .output();

    let out = match out {
        Ok(o) => o,
        Err(_) => return vec![],
    };

    let text = String::from_utf8_lossy(&out.stdout);
    let mut versions = vec![];

    for line in text.lines() {
        if let Some(ver) = line.strip_prefix("Version: ") {
            let v = ver.trim().to_string();
            if !versions.contains(&v) {
                versions.push(v);
            }
        }
    }

    versions
}
