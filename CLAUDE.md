# CLAUDE.md

This file provides guidance to Claude Code when working with this repository.

## What is QWERT

QWERT is a **dev environment manager** — not a package manager. The user declares what their machine should have in `$QWERT_CONFIG_DIR/qwert.yml`, saves that directory in a personal repo, and runs `qwert apply` on any new machine to replicate the environment exactly.

- `~/.config/qwert/` (`QWERT_CONFIG_DIR`) — the developer's personal dotfiles. Free-form directory, saved in a personal repo.
- `~/.qwert/` — qwert installation (binary, recipe cache, state). Never edited by the user.

## Architecture (v2 — Rust)

The current implementation is in Rust (`src/`). The original shell scripts are preserved in `v1/` with shims at the repo root.

```
src/
├── main.rs
├── cli.rs                  ← clap subcommands
├── commands/               ← one file per command
│   ├── apply.rs            ← installs declared tools, uninstalls orphans
│   ├── use_cmd.rs          ← adds tool to qwert.yml and installs
│   ├── drop_cmd.rs         ← removes tool from qwert.yml
│   ├── status.rs
│   ├── search.rs           ← searches qwert recipes + brew
│   ├── list.rs
│   ├── upgrade.rs
│   ├── reinstall.rs
│   ├── update.rs
│   ├── doctor.rs
│   ├── config.rs
│   └── help.rs
├── recipe/
│   ├── schema.rs           ← Recipe, RecipeMeta, RecipeKind, Commands
│   ├── index.rs            ← loads recipes from ~/.qwert/recipes/
│   └── runner.rs           ← installs/upgrades/uninstalls recipes
├── adapters/               ← package manager adapters
│   ├── mod.rs              ← PackageAdapter trait + for_kind()
│   ├── brew.rs
│   ├── apt.rs
│   └── pacman.rs
├── config/
│   ├── qwert_yml.rs        ← reads/writes qwert.yml
│   └── state_yml.rs        ← tracks what qwert installed (~/.qwert/state.yml)
├── platform/
│   └── mod.rs              ← Platform enum, PlatformOps trait, detect(), which()
└── ui/
    ├── printer.rs           ← ok/installing/failed/search_result/command
    └── colors.rs
```

## Commands

```
qwert use <tool>             # add to qwert.yml and install
qwert use <tool> --no-install
qwert drop <tool>            # remove from qwert.yml
qwert drop <tool> --uninstall
qwert apply                  # install declared tools, uninstall orphans
qwert apply <tool>
qwert status / status <tool>
qwert search <term>          # searches recipes + brew
qwert list
qwert upgrade / upgrade <tool>
qwert reinstall <tool>
qwert update
qwert doctor
qwert config edit
qwert help
```

## Recipe System

Recipes are TOML files in `recipes/` (cached to `~/.qwert/recipes/` at install time).

### Types

| Type | Behavior |
|------|----------|
| `brew` | BrewAdapter handles install/upgrade/uninstall automatically from `meta.name` |
| `apt` | AptAdapter — same pattern |
| `pacman` | PacmanAdapter — same pattern |
| `qwert` | Custom commands defined in `[install]`, `[upgrade]`, `[uninstall]` sections |

### Schema

```toml
[meta]
name = "tmux"
version = "1.0.0"
description = "Terminal multiplexer"
type = "brew"          # brew | apt | pacman | qwert
depends = []           # other recipe names to install first
pkg = "git-delta"      # optional: override package name passed to adapter (default: meta.name)

[check]
command = "tmux"
version_flag = "-V"

# Only needed for type = "qwert" or cross-platform fallback
[install]
macos = "custom install command"
debian = ["step one", "step two"]

[config]
dest = "~/.tmux.conf"
symlink = true
```

### Adapter pattern

For `brew`/`apt`/`pacman` recipes, **do not write `[install]`/`[upgrade]`/`[uninstall]` sections** — the adapter derives commands from `meta.name` (or `meta.pkg` if set). Explicit sections are only for `qwert` type or platform fallback.

The runner tries the adapter first; if unavailable, falls back to explicit commands.

## State tracking

`~/.qwert/state.yml` records which tools qwert has installed. Used by `apply` to detect orphans (tools removed from `qwert.yml` since last apply) and uninstall them.

## Platform detection

`platform::detect()` returns `Platform::MacOS`, `Platform::Debian`, or `Platform::Unknown`.
Detection: `cfg!(target_os = "macos")` for macOS; `/usr/bin/apt-get` for Debian.

## Testing

Follow `.project/ai/commands/test.md` for test conventions.

**Test files are separate from source files** using Rust's `#[path]` attribute:

```
src/adapters/brew.rs          ← source
src/adapters/tests/brew.rs    ← tests
```

Link from the source file:
```rust
#[cfg(test)]
#[path = "tests/brew.rs"]
mod tests;
```

All tests follow the triple-A pattern (`// arrange`, `// act`, `// assert`).

Run tests: `make t`

## Development

```bash
make t        # cargo test
make build    # cargo build --release
```

Dependencies: `clap 4`, `serde + serde_yml`, `toml`, `anyhow`, `dirs`

## v1 (shell scripts)

The original shell implementation is in `v1/`. Root-level `bin/qwert` and `run` are shims that delegate to `v1/`. This keeps existing `~/.qwert` installations working while v2 is developed.
