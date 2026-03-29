#!/bin/bash

ITERM_CONFIG_DIR="$HOME/.config/qwert/iterm2"

mkdir -p "$ITERM_CONFIG_DIR"

# Configurar iTerm2 para ler/escrever direto no custom folder (sem passar pelo cfprefsd)
defaults write com.googlecode.iterm2 PrefsCustomFolder -string "$ITERM_CONFIG_DIR"
defaults write com.googlecode.iterm2 LoadPrefsFromCustomFolder -bool true

echo "  - iTerm2 configurado para usar $ITERM_CONFIG_DIR"
