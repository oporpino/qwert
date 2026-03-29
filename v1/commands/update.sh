#!/bin/bash

QWERT_REF=${1:-latest}

if [ "$QWERT_REF" = "latest" ]; then
    QWERT_REF=$(curl -fsSL "https://api.github.com/repos/gporpino/qwert/tags" \
        | grep '"name"' | sed 's/.*"name": *"\(.*\)".*/\1/' | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$' | head -1)
    if [ -z "$QWERT_REF" ]; then
        QWERT_REF="main"
    fi
fi

QWERT_INSTALL_URL="https://raw.githubusercontent.com/gporpino/qwert/${QWERT_REF}/scripts/install.sh"

QWERT_SILENT=${QWERT_SILENT:-0}
if [ "$QWERT_SILENT" = "0" ]; then
    QWERT_CURRENT=$(cat "$HOME/.qwert/version" 2>/dev/null || echo "unknown")
    echo "> Updating QWERT from ${QWERT_CURRENT} to ${QWERT_REF}..."
fi

QWERT_VERSION="$QWERT_REF" QWERT_FORCE="1" sh -c "$(curl -fsSL "$QWERT_INSTALL_URL")"

unset QWERT_REF QWERT_INSTALL_URL
