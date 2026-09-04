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

/// Pad `s` to `w` visible columns, truncating when longer so the next column
/// stays aligned. Truncation uses an ellipsis to hint at the cut.
fn pad(s: &str, w: usize) -> String {
    let n = s.chars().count();
    if n > w {
        if w == 0 {
            return String::new();
        }
        let mut s: Vec<char> = s.chars().collect();
        s.truncate(w.saturating_sub(1));
        let mut out: String = s.into_iter().collect();
        out.push('…');
        return out;
    }
    format!("{}{}", s, " ".repeat(w - n))
}

/// Maximum visible width per column (NAME, STATUS, TYPE, SETUP, ORIGIN, VERSION, PROFILE).
const COL_MAX: [usize; 7] = [16, 42, 8, 14, 8, 12, 20];

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

    let mut rows: Vec<Row> = Vec::new();

    for name in &names {
        let version = config.version_of(&profile, name).to_string();
        let profiles = if all {
            config.profiles_of_tool(name).join(",")
        } else {
            profile.clone()
        };

        let recipe = index::find(name, &recipes_dir);
        let source = config
            .config_source_for(&profile, name)
            .map(|s| std::path::PathBuf::from(qwert_yml::expand_tilde(s)));
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
                    .map(|s| runner::setup_status_label(s, source.as_deref()).to_string())
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

    // Column widths (visible char count), capped so long content is truncated
    // instead of pushing the next column out of alignment.
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
    for i in 0..7 {
        w[i] = w[i].min(COL_MAX[i]);
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
            // Truncate the kind label to the column width, keeping the ANSI tag
            // around only the visible portion.
            let kind_vis = r.kind.chars().count() + 2;
            let (label, extra) = if kind_vis > w[2] {
                // width cannot hold even one tag — fall back to plain text
                (pad(&r.kind, w[2]), String::new())
            } else {
                (r.kind.clone(), " ".repeat(w[2] - kind_vis))
            };
            format!("{}{}", printer::kind_tag(&label), extra)
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
#[cfg(test)]
mod tests {
    use super::pad;

    #[test]
    fn pad_adds_spaces_when_shorter() {
        // arrange
        let s = "abc";
        // act
        let out = pad(s, 6);
        // assert
        assert_eq!(out, "abc   ");
    }

    #[test]
    fn pad_leaves_exact_fit_unchanged() {
        // arrange
        let s = "abcdef";
        // act
        let out = pad(s, 6);
        // assert
        assert_eq!(out, "abcdef");
    }

    #[test]
    fn pad_truncates_with_ellipsis_when_longer() {
        // arrange
        let s = "installed (OpenClaw 2026.7.1 (2d2ddc4))";
        // act
        let out = pad(s, 20);
        // assert — widt-1 chars + ellipsis
        assert_eq!(out.chars().count(), 20);
        assert!(out.ends_with('…'));
        assert!(out.starts_with("installed "));
    }

    #[test]
    fn pad_zero_width_returns_empty() {
        // arrange
        let s = "abc";
        // act
        let out = pad(s, 0);
        // assert
        assert_eq!(out, "");
    }

    #[test]
    fn pad_width_one_truncates_to_ellipsis() {
        // arrange
        let s = "abc";
        // act
        let out = pad(s, 1);
        // assert
        assert_eq!(out, "…");
    }
}
