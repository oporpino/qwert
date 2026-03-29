# CLAUDE.md

This file provides guidance to Claude Code when working with this repository.

## What is QWERT

QWERT is a **dev environment manager** — not a package manager. The user declares what their machine should have in `~/.config/qwert.yml`, saves their dotfiles in a personal repo, and runs `qwert apply` on any new machine to replicate the environment exactly.

- `~/.config/qwert.yml` — the manifest: tools, stacks, hooks. Managed by qwert.
- `~/.config/qwert/` — the developer's personal dotfiles. Free-form directory, saved in a personal repo.
- `~/.qwert/` — qwert installation (binary, recipes, completions, state). Never edited by the user.

No env vars needed. No shell config beyond what the installer writes.

## Architecture (v2 — Rust)

The current implementation is in Rust (`src/`). The original shell scripts are preserved in `v1/` with shims at the repo root.

```
src/
├── main.rs
├── cli.rs                  ← clap subcommands
├── commands/               ← one file per command
│   ├── apply.rs            ← installs + sets up declared tools, uninstalls orphans
│   ├── use_cmd.rs          ← declare + install + setup
│   ├── install_cmd.rs      ← declare + install only
│   ├── setup_cmd.rs        ← declare + run setup only
│   ├── uninstall_cmd.rs    ← remove from yml + uninstall
│   ├── drop_cmd.rs         ← full teardown: uninstall + undo setup
│   ├── status.rs
│   ├── info.rs             ← recipe details + install/setup status
│   ├── search.rs           ← searches qwert recipes + brew
│   ├── list.rs
│   ├── upgrade.rs
│   ├── reinstall.rs
│   ├── hook.rs             ← outputs shell hooks (init/end)
│   ├── completions.rs      ← outputs shell completion script
│   ├── self_cmd.rs         ← self upgrade/reinstall
│   ├── recipes_cmd.rs      ← recipes update
│   ├── doctor.rs
│   ├── config.rs
│   └── help.rs
├── recipe/
│   ├── schema.rs           ← Recipe, RecipeMeta, RecipeKind, RecipeSetup, Commands
│   ├── index.rs            ← find/load_all from ~/.qwert/recipes/<name>/
│   └── runner.rs           ← install/upgrade/uninstall/setup/undo_setup
├── adapters/               ← package manager adapters
│   ├── mod.rs              ← PackageAdapter trait + for_kind()
│   ├── brew.rs
│   ├── apt.rs
│   └── pacman.rs
├── config/
│   ├── qwert_yml.rs        ← reads/writes qwert.yml
│   └── state_yml.rs        ← tracks what qwert installed (~/.qwert/state.yml)
├── platform/
│   ├── mod.rs              ← Platform enum, detect(), which(), run_cmd()
│   └── fs.rs               ← create_symlink(), copy_file()
└── ui/
    ├── printer.rs           ← ok/installing/failed/search_result/command
    └── colors.rs
```

## Commands

```
qwert use <tool>             # declare + install + setup
qwert use <tool> --no-install
qwert install <tool>         # declare + install (no setup)
qwert setup <tool>           # declare + run setup
qwert uninstall <tool>       # remove from qwert.yml + uninstall
qwert drop <tool>            # full teardown: uninstall + undo setup (with backup)
qwert apply                  # install + setup all declared tools, uninstall orphans
qwert apply <tool>
qwert status / status <tool>
qwert info <tool>            # recipe details, install status, setup status
qwert search <term>          # searches recipes + brew
qwert list
qwert upgrade / upgrade <tool>
qwert upgrade --all
qwert reinstall <tool>
qwert self upgrade
qwert self reinstall
qwert recipes update
qwert hook init / hook end   # output shell hooks (eval'd in .zshrc)
qwert completions <shell>    # output completion script
qwert doctor
qwert config edit
qwert help
```

## Recipe System

Recipes live in `recipes/<name>/` (cached to `~/.qwert/recipes/` at install time). Each recipe is a directory with up to two files — both optional:

```
recipes/
└── tmux/
    ├── install.toml   ← install/upgrade/uninstall + meta
    └── setup.toml     ← symlinks, copies, or commands for config setup
```

If only `setup.toml` exists, qwert synthesizes meta from the directory name and uses the platform default adapter. If neither file exists, qwert falls back to `brew install <name>` / `apt install <name>`.

### Types

| Type | Behavior |
|------|----------|
| `brew` | BrewAdapter handles install/upgrade/uninstall from `meta.name` |
| `apt` | AptAdapter — same pattern |
| `pacman` | PacmanAdapter — same pattern |
| `qwert` | Custom commands in `[install]`, `[upgrade]`, `[uninstall]` sections |

### `install.toml`

```toml
[meta]
name = "tmux"
version = "1.0.0"
description = "Terminal multiplexer"
type = "brew"          # brew | apt | pacman | qwert
depends = []           # other recipe names to install first
pkg = "git-delta"      # optional: override package name (default: meta.name)

[check]
command = "tmux"
version_flag = "-V"

# Only needed for type = "qwert" or cross-platform fallback
[install]
macos = "custom install command"
debian = ["step one", "step two"]
```

### `setup.toml`

```toml
# symlink: ~/.tmux.conf → ~/.config/qwert/tmux (undo = remove symlink)
dest = "~/.tmux.conf"
symlink = true
# src optional — defaults to ~/.config/qwert/<name>

# commands: run on setup (undo = [undo] section)
dest = "~/.config/qwert/iterm2"
macos = ["defaults write com.googlecode.iterm2 PrefsCustomFolder -string ~/.config/qwert/iterm2"]

[undo]
macos = ["defaults delete com.googlecode.iterm2 PrefsCustomFolder"]
```

**Setup types and undo behaviour:**
- `symlink = true` — undo removes the symlink
- copy (dest exists, no symlink) — undo backs up to `~/.qwert/backups/<name>/` then removes
- commands — undo runs `[undo]` section; warns if not defined

### Adapter pattern

For `brew`/`apt`/`pacman` recipes, **do not write `[install]`/`[upgrade]`/`[uninstall]` sections** — the adapter derives commands from `meta.name` (or `meta.pkg` if set). Explicit sections are only for `qwert` type or platform fallback.

## qwert.yml schema

```yaml
tools:
  - tmux
  - lvim

hooks:
  init:
    - ~/.config/qwert/zsh/init.sh
  end:
    - ~/.config/qwert/zsh/end.sh
```

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
