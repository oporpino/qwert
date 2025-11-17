# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

QWERT is a dev tools manager for setting up development environments on macOS (Linux support planned). It automates the installation and configuration of tools like Homebrew, LunarVim, Tmux, Neovim, and iTerm2.

## Architecture

### Core Components

- **`bin/qwert`**: Main entry point CLI that dispatches commands to scripts
- **`run`**: Shell script that manages PATH and sources custom zsh files from `~/.config/qwert/zsh/`
- **`scripts/install.sh`**: Installation script that sets up Oh-My-Zsh, clones QWERT to `~/.qwert`, and configures `.zshrc`
- **`scripts/init.sh`**: Initialization script sourced by the `run` script
- **`scripts/completions.sh`**: Bash completion definitions for the `qwert` command

### Command Structure

Commands follow a two-level structure: `qwert <command> <subcommand>`

- **setup**: Install and configure tools
  - Implementation: `commands/setup.sh` dispatches to platform-specific scripts
  - `macos`: Runs `commands/setup/macos/all.sh` which sequentially sources setup scripts for homebrew, neovim, lvim, tmux, wemux, iterm2, and config
  - `linux`: Not yet implemented

- **status**: Check installation status
  - Implementation: `commands/status.sh`
  - Subcommands: `all`, `homebrew` (not yet implemented)

### Configuration Management

QWERT uses a two-tier config system:

1. **Defaults**: `defaults/config/` contains default configurations for:
   - `zsh/init.sh`: Custom zsh initialization
   - `tmux/tmux.conf`: Tmux configuration
   - `lvim/config.lua`: LunarVim configuration
   - `iterm2/setup.sh`: iTerm2 setup script

2. **User configs**: Copied to `~/.config/qwert/` on setup and symlinked to their expected locations (e.g., `~/.tmux.conf` → `~/.config/qwert/tmux/tmux.conf`)

This allows users to customize their environment by modifying files in `~/.config/qwert/` and backing them up separately.

### Environment Variables

- `QWERT_DIR`: Set to `$HOME/.qwert`, used throughout scripts to reference the installation directory
- `QWERT_VERBOSE`: Set to `1` to enable verbose output during initialization

## Common Commands

### Installation
```bash
sh -c "$(curl -fsSL https://raw.githubusercontent.com/gporpino/qwert/v0.1.1/scripts/install.sh)"
```

### Setup Environment (macOS)
```bash
qwert setup macos
```

After setup, run these manual steps:
- Open LunarVim and run `:PackerSync` to install packages
- Open tmux and press `<prefix>+I` to install plugins

### Check Status
```bash
qwert status all
```

## Development Notes

- All scripts use `#!/bin/bash` shebang
- Scripts clean up environment variables with `unset` at the end
- The `run` script manages PATH by removing any existing `$QWERT_DIR/bin` entries and re-adding it to the front
- Installation script modifies `.zshrc` to source QWERT init at the beginning and end files, plus adds reload alias and completions
- Setup scripts check for existing installations before proceeding and warn users if tools are already installed
