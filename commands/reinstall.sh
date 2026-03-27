#!/bin/bash

QWERT_CONFIG_VERSION_FILE=$HOME/.config/qwert/version

if [ ! -f "$QWERT_CONFIG_VERSION_FILE" ]; then
    echo "  - [error] No version file found at $QWERT_CONFIG_VERSION_FILE"
    exit 1
fi

QWERT_VERSION=$(cat "$QWERT_CONFIG_VERSION_FILE")

echo "> Reinstalling QWERT version ${QWERT_VERSION}..."

$QWERT_DIR/commands/update.sh "$QWERT_VERSION"

unset QWERT_CONFIG_VERSION_FILE QWERT_VERSION
