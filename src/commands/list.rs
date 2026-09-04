use anyhow::Result;

use crate::config::qwert_yml;
use crate::recipe::{index, runner};
use crate::ui::printer;

struct Row {
    name: String,
    status: String,
    ok: bool,
    kind: String,
    setup: String,
    origin: String,
    version: String,
    profiles: String,
}

fn pad(s: &str, w: usize) -> String {
    let n = s.chars().count();
    if n >= w {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(w - n))
    }
}

pub fn run(all: bool) -> Result<()> {
    let manifest_path = qwert_yml::manifest_path();
    let config = qwert_yml::QwertConfig::load(&manifest_path)?;
    let profile = crate::config::machine::MachineIdentity::load()?.active_profile().to_string();

    // Without --all: only the active profile's tools. With --all: union across profiles.
    let mut names: Vec<String> = Vec::new();
    if all {
        for p in config.profile_names() {
            for n in config.tool_names_for_profile(&p) {
                if !names.contains(&n) {
                    names.push(n);
                }
            }
        }
    } else {
        names = config.tool_names_for_profile(&profile);
    }

    if names.is_empty() {
        printer::info("No tools declared. Run `qwert use <tool>` to add one.");
        return Ok(());
    }

    let recipes_dir = index::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;
    let config_dir = qwert_yml::config_dir();

    let mut rows: Vec<Row> = Vec::new();

    for name in &names {
        let version = config.version_of(&profile, name).to_string();
        let profiles = if all {
            config.profiles_of_tool(name).join(",")
        } else {
            profile.clone()
        };

        let recipe = index::find(name, &recipes_dir);
        match &recipe {
            Some(recipe) => {
                let setup_only = recipe.setup_only;
                let origin = if recipe.local {
                    "[local]".to_string()
                } else {
                    "[remote]".to_string()
                };
                let setup = recipe
                    .setup
                    .as_ref()
                    .map(|s| runner::setup_status_label(s, &config_dir, name).to_string())
                    .unwrap_or_else(|| "—".to_string());

                let (status, ok, kind) = if setup_only {
                    ("n/a".to_string(), true, "—".to_string())
                } else {
                    let kind = recipe.meta.kind.to_string();
                    if runner::is_installed(recipe) {
                        (runner::version_msg("installed", runner::installed_version(recipe)), true, kind)
                    } else {
                        ("not installed".to_string(), false, kind)
                    }
                };
                rows.push(Row { name: name.clone(), status, ok, kind, setup, origin, version, profiles });
            }
            None => {
                // No recipe — status from the platform, no type/origin.
                let installed = crate::platform::which(name);
                let status = if installed {
                    format!("installed{}", crate::platform::version_of(name, "--version")
                        .map(|v| format!(" ({})", v))
                        .unwrap_or_default())
                } else {
                    "not installed".to_string()
                };
                rows.push(Row {
                    name: name.clone(),
                    status,
                    ok: installed,
                    kind: "—".to_string(),
                    setup: "—".to_string(),
                    origin: "—".to_string(),
                    version,
                    profiles,
                });
            }
        }
    }

    // Column widths (visible char count).
    let mut w = [0usize; 7];
    for r in &rows {
        w[0] = w[0].max(r.name.chars().count());
        w[1] = w[1].max(r.status.chars().count());
        w[2] = w[2].max(r.kind.chars().count() + 2);
        w[3] = w[3].max(r.setup.chars().count());
        w[4] = w[4].max(r.origin.chars().count());
        w[5] = w[5].max(r.version.chars().count());
        w[6] = w[6].max(r.profiles.chars().count());
    }
    w[0] = w[0].max(4);
    w[1] = w[1].max(6);
    w[2] = w[2].max(4);
    w[3] = w[3].max(5);
    w[4] = w[4].max(6);
    w[5] = w[5].max(7);
    w[6] = w[6].max(7);

    let header = [
        "NAME", "STATUS", "TYPE", "SETUP", "ORIGIN", "VERSION", "PROFILE",
    ];

    printer::blank();
    if all {
        printer::info(&format!("all profiles ({})", config.profile_names().join(", ")));
    } else {
        printer::info(&format!("profile: {}", profile));
    }
    printer::blank();

    // Header row.
    let mut head = String::new();
    for (i, h) in header.iter().enumerate() {
        if i == 0 {
            head.push_str(&printer::bold_text(&pad(h, w[i])));
        } else {
            head.push(' ');
            head.push(' ');
            head.push_str(&printer::bold_text(&pad(h, w[i])));
        }
        if i == 6 {
            break;
        }
    }
    println!("{}", head);
    let rule = format!("  {}", "—".repeat(w.iter().sum::<usize>() + 14));
    println!("{}", printer::dim_text(&rule));

    // Data rows.
    for r in &rows {
        let name = printer::bold_text(&pad(&r.name, w[0]));
        let status = {
            let p = pad(&r.status, w[1]);
            if r.ok { printer::success_text(&p) } else { printer::error_text(&p) }
        };
        let kind = {
            let tag = printer::kind_tag(&r.kind);
            let visible = r.kind.chars().count() + 2;
            let px = w[2].saturating_sub(visible);
            format!("{}{}", tag, " ".repeat(px))
        };
        let setup = printer::dim_text(&pad(&r.setup, w[3]));
        let origin = {
            let p = pad(&r.origin, w[4]);
            if r.origin == "[local]" {
                printer::success_text(&p)
            } else {
                printer::dim_text(&p)
            }
        };
        let version = printer::dim_text(&pad(&r.version, w[5]));
        let profiles = pad(&r.profiles, w[6]);

        println!(
            "  {}  {}  {}  {}  {}  {}  {}",
            name, status, kind, setup, origin, version, profiles
        );
    }

    printer::blank();
    Ok(())
}