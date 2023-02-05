#!/bin/bash

alias reload!="source ~/.zshrc"

if [ -z $TMUX ]; then
    if ! $(exec tmux attach -t qwert); then
        exec tmux new -s qwert
    fi
fi

