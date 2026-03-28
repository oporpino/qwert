#!/bin/bash

ITERM_CONFIG_DIR="$HOME/.config/qwert/iterm2"
ITERM_PREFS_FILE="com.googlecode.iterm2.plist"
ITERM_DEFAULT_PREFS="$HOME/Library/Preferences/$ITERM_PREFS_FILE"
ITERM_QWERT_PREFS="$ITERM_CONFIG_DIR/$ITERM_PREFS_FILE"

# Criar diretório de configuração se não existir
mkdir -p "$ITERM_CONFIG_DIR"

# Fazer backup e criar symlink
if [ ! -L "$ITERM_DEFAULT_PREFS" ]; then
    if [ -f "$ITERM_DEFAULT_PREFS" ]; then
        echo "  - Backup de preferências existentes em ${ITERM_DEFAULT_PREFS}.bkp"
        mv "$ITERM_DEFAULT_PREFS" "${ITERM_DEFAULT_PREFS}.bkp"

        if [ ! -f "$ITERM_QWERT_PREFS" ]; then
            cp "${ITERM_DEFAULT_PREFS}.bkp" "$ITERM_QWERT_PREFS"
        fi
    fi

    echo "  - Criando symlink $ITERM_DEFAULT_PREFS -> $ITERM_QWERT_PREFS"
    ln -s "$ITERM_QWERT_PREFS" "$ITERM_DEFAULT_PREFS"
fi

echo "  - iTerm2 configurado via symlink em $ITERM_DEFAULT_PREFS"
