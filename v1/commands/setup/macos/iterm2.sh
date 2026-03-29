#!/bin/bash

# Check if iTerm2 is installed
if [ -d "/Applications/iTerm.app" ]; then
    echo "  - iTerm2 is already installed."
else
    echo "  - [warn] iTerm2 is not installed. Please install iTerm2 from https://iterm2.com/"
    echo "  - [warn] Skipping iTerm2 configuration."
    exit 0
fi
