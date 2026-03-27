#!/bin/bash

QWERT_VERSION_FILE=$HOME/.qwert/version

if [ ! -f "$QWERT_VERSION_FILE" ]; then
    echo "unknown"
else
    cat "$QWERT_VERSION_FILE"
fi

unset QWERT_VERSION_FILE
