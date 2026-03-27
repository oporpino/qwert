#!/bin/bash

QWERT_REPO="https://github.com/gporpino/qwert.git"
QWERT_DIR=$HOME/.qwert
QWERT_TMP="${QWERT_DIR}_tmp"

echo "> Updating QWERT from main..."

# Clone latest into a temp dir
if ! git clone --depth 1 "$QWERT_REPO" "$QWERT_TMP"; then
    echo "  - [error] Failed to clone QWERT repository"
    exit 1
fi

# Replace installation, preserving nothing (no user config lives here)
rm -rf "$QWERT_DIR"
mv "$QWERT_TMP" "$QWERT_DIR"
rm -rf "$QWERT_DIR/.git"

echo "> QWERT updated successfully"
echo "  - Reload your shell to apply changes: source ~/.zshrc"

unset QWERT_REPO QWERT_DIR QWERT_TMP
