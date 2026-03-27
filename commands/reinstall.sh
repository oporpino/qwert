#!/bin/bash

QWERT_VERSION_FILE=$HOME/.qwert/version

if [ ! -f "$QWERT_VERSION_FILE" ]; then
    echo "  - [warn] No version file found, using main"
    QWERT_VERSION="main"
else
    QWERT_VERSION=$(cat "$QWERT_VERSION_FILE")
fi

echo "> Reinstalling QWERT version ${QWERT_VERSION}..."

$QWERT_DIR/commands/update.sh "$QWERT_VERSION"

unset QWERT_VERSION_FILE QWERT_VERSION
