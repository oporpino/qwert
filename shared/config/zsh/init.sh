#!/bin/bash

# run tmux in silent mode
if [ -z $TMUX ]; then
    if ! $(exec tmux attach -t qwert > /dev/null 2>&1); then
        exec tmux new -s qwert
    fi
fi

alias reload!=${source ~/.zshrc}
