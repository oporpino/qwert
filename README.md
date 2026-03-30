# QWERT

A dev environment manager. Declare the tools you need in `~/.qwert/config.yml`, save your dotfiles in a personal repo, and run `qwert apply` on any new machine to replicate the environment exactly.

## Install

```sh
sh -c "$(curl -fsSL https://raw.githubusercontent.com/oporpino/qwert/latest/scripts/install.sh)"
```

## Usage

```
qwert use <tool>         # declare + install + setup
qwert install <tool>     # declare + install (no setup)
qwert setup <tool>       # declare + run setup
qwert uninstall <tool>   # remove from config + uninstall
qwert drop <tool>        # full teardown: uninstall + undo setup
qwert apply              # sync all declared tools
qwert status             # show installed tools
qwert search <term>      # search recipes + brew
qwert upgrade <tool>     # upgrade a tool
qwert upgrade --all      # upgrade all tools
qwert self upgrade       # upgrade qwert itself
qwert doctor             # check environment health
```

## Config

`~/.qwert/config.yml` — the manifest for your environment:

```yaml
tools:
  - tmux
  - lvim

hooks:
  before:
    - ~/.qwert/zsh/init.sh
  init:
    - ~/.qwert/zsh/end.sh
```

Save `~/.qwert/` in a private repository. On a new machine, clone it and run `qwert apply`.

## How it works

- `~/.qwert/` — your dotfiles. Free-form, version-controlled in your personal repo.
- `~/.local/share/qwert/` — qwert runtime data (recipes, state, backups). Never edited manually.
- `/opt/qwert/bin/qwert` — the binary.

Recipes live in `~/.local/share/qwert/recipes/`. Each recipe can define install steps, setup (symlinks, copies, commands), and undo behaviour.
