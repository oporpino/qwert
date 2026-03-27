#!/bin/bash

QWERT_REF=${1:-latest}

if [ "$QWERT_REF" = "latest" ]; then
    QWERT_REF=$(curl -fsSL "https://api.github.com/repos/gporpino/qwert/releases/latest" \
        | grep '"tag_name"' | sed 's/.*"tag_name": *"\(.*\)".*/\1/')
    if [ -z "$QWERT_REF" ]; then
        QWERT_REF="main"
    fi
fi

QWERT_INSTALL_URL="https://raw.githubusercontent.com/gporpino/qwert/${QWERT_REF}/scripts/install.sh"

echo "> Updating QWERT from ${QWERT_REF}..."

QWERT_VERSION="$QWERT_REF" sh -c "$(curl -fsSL "$QWERT_INSTALL_URL")"

unset QWERT_REF QWERT_INSTALL_URL
