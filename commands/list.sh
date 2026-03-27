#!/bin/bash

curl -fsSL "https://api.github.com/repos/gporpino/qwert/tags" \
    | grep '"name"' \
    | sed 's/.*"name": *"\(.*\)".*/\1/' \
    | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$'
