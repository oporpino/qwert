use anyhow::Result;

use crate::config::qwert_yml;
use crate::recipe::{index, runner};
use crate::ui::printer;

struct Row {
    name: String,
    status: String,
    ok: bool,
    manager: String,
    setup_ok: bool,
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

/// Maximum visible width per column (NAME, STATUS, MANAGER, SETUP, ORIGIN, VERSION, PROFILE).
const COL_MAX: [usize; 7] = [16, 14, 13, 14, 8, 28, 20];

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
                let origin = if recipe.local {
                    "[local]".to_string()
                } else {
                    "[remote]".to_string()
                };
                // Setup is "done" only when the recipe has a setup section and it
                // is in a settled state (configured/linked/copied).
                let setup_ok = recipe
                    .setup
                    .as_ref()
                    .map(|s| {
                        matches!(runner::setup_status_label(s, source.as_deref()), "configured" | "linked" | "copied")
                    })
                    .unwrap_or(false);

                // setup_only → "config only"; any recipe present → managed by qwert
                let manager = if recipe.setup_only {
                    "config only".to_string()
                } else {
                    "qwert".to_string()
                };

                let (status, ok, version) = if recipe.setup_only {
                    ("n/a".to_string(), true, "—".to_string())
                } else if runner::is_installed(recipe) {
                    (
                        "installed".to_string(),
                        true,
                        runner::installed_version(recipe).unwrap_or_else(|| "—".to_string()),
                    )
                } else {
                    ("not installed".to_string(), false, "—".to_string())
                };
                rows.push(Row { name: name.clone(), status, ok, manager, setup_ok, origin, version, profiles });
            }
            None => {
                // No recipe — managed by yuiop (the platform's package manager).
                // Ask yuiop for the authoritative status; `which` covers tools
                // yuiop does not know.
                let installed = crate::adapters::yuiop::status(name)
                    .unwrap_or_else(|| crate::platform::which(name));
                let version = if installed {
                    crate::platform::version_of(name, "--version").unwrap_or_else(|| "—".to_string())
                } else {
                    "—".to_string()
                };
                let status = if installed { "installed".to_string() } else { "not installed".to_string() };
                rows.push(Row {
                    name: name.clone(),
                    status,
                    ok: installed,
                    manager: "yuiop".to_string(),
                    setup_ok: false,
                    origin: "—".to_string(),
                    version,
                    profiles,
                });
            }
        }
    }

    // Columns: NAME STATUS MANAGER SETUP ORIGIN VERSION, plus PROFILE only with --all.
    let ncols = if all { 7 } else { 6 };
    let header = if all {
        ["NAME", "STATUS", "MANAGER", "SETUP", "ORIGIN", "VERSION", "PROFILE"].to_vec()
    } else {
        ["NAME", "STATUS", "MANAGER", "SETUP", "ORIGIN", "VERSION"].to_vec()
    };
    // Column widths (visible char count), driven by the largest content in each
    // column, capped so long content is truncated instead of pushing the next
    // column out of alignment.
    let mut w = [0usize; 7];
    for r in &rows {
        w[0] = w[0].max(r.name.chars().count());
        w[1] = w[1].max(r.status.chars().count());
        w[2] = w[2].max(r.manager.chars().count() + 2);
        w[3] = w[3].max(if r.setup_ok { 4 } else { 1 });
        w[4] = w[4].max(r.origin.chars().count());
        w[5] = w[5].max(r.version.chars().count());
        w[6] = w[6].max(r.profiles.chars().count());
    }
    for i in 0..7 {
        w[i] = w[i].min(COL_MAX[i]);
    }
    for i in 0..ncols {
        w[i] = w[i].max(header[i].chars().count());
    }

    printer::blank();
    if all {
        printer::info(&format!("all profiles ({})", config.profile_names().join(", ")));
    } else {
        printer::info(&format!("profile: {}", profile));
    }
    printer::blank();

    // Header row — same 2-space indent as the data rows.
    let mut head = String::new();
    for i in 0..ncols {
        head.push_str("  ");
        head.push_str(&printer::bold_text(&pad(&header[i], w[i])));
    }
    println!("{}", head);
    let rule = format!("  {}", "—".repeat(w[..ncols].iter().sum::<usize>() + ncols * 2));
    println!("{}", printer::orange_text(&rule));

    // Data rows.
    for r in &rows {
        let name = printer::bold_text(&pad(&r.name, w[0]));
        let status = {
            let p = pad(&r.status, w[1]);
            if r.ok { printer::success_text(&p) } else { printer::error_text(&p) }
        };
        let kind = {
            // Truncate the manager label to the column width, keeping the ANSI tag
            // around only the visible portion.
            let kind_vis = r.manager.chars().count() + 2;
            let (label, extra) = if kind_vis > w[2] {
                // width cannot hold even one tag — fall back to plain text
                (pad(&r.manager, w[2]), String::new())
            } else {
                (r.manager.clone(), " ".repeat(w[2] - kind_vis))
            };
            format!("{}{}", printer::kind_tag(&label), extra)
        };
        let setup = if r.setup_ok {
            printer::success_text(&pad("done", w[3]))
        } else {
            " ".repeat(w[3])
        };
        let origin = {
            let p = pad(&r.origin, w[4]);
            if r.origin == "[local]" {
                printer::success_text(&p)
            } else {
                printer::orange_text(&p)
            }
        };
        let version = pad(&r.version, w[5]);

        let mut line = format!("  {}  {}  {}  {}  {}  {}", name, status, kind, setup, origin, version);
        if all {
            line.push_str(&format!("  {}", pad(&r.profiles, w[6])));
        }
        println!("{}", line);
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
