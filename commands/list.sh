#!/bin/bash

curl -fsSL "https://api.github.com/repos/gporpino/qwert/releases" \
    | grep '"tag_name"' \
    | sed 's/.*"tag_name": *"\(.*\)".*/\1/'

unset QWERT_LATEST
