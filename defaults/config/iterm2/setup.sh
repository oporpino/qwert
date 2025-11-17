#!/bin/bash

ITERM_CONFIG_DIR="$HOME/.config/qwert/iterm2"
ITERM_PREFS_FILE="com.googlecode.iterm2.plist"
ITERM_DEFAULT_PREFS="$HOME/Library/Preferences/$ITERM_PREFS_FILE"

# Criar diretório de configuração se não existir
mkdir -p "$ITERM_CONFIG_DIR"

# Mover arquivo de preferências existente para o diretório custom
if [ -f "$ITERM_DEFAULT_PREFS" ] && [ ! -f "$ITERM_CONFIG_DIR/$ITERM_PREFS_FILE" ]; then
    echo "  - Movendo preferências existentes para $ITERM_CONFIG_DIR"
    cp "$ITERM_DEFAULT_PREFS" "$ITERM_CONFIG_DIR/$ITERM_PREFS_FILE"
fi

# Configurar iTerm2 para usar diretório custom
defaults write com.googlecode.iterm2 PrefsCustomFolder -string "$ITERM_CONFIG_DIR"
defaults write com.googlecode.iterm2 LoadPrefsFromCustomFolder -bool true

echo "✓ iTerm2 configurado para usar $ITERM_CONFIG_DIR"
