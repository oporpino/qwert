#!/bin/bash

# config custom zsh
QWERT_ZSH_INIT=$QWERT_DIR/shared/config/zsh/init.sh

CONFIG_DIR=$HOME/.config
QWERT_ZSH_INIT_CUSTOM=$CONFIG_DIR/qwert/zsh/init.sh

# creating config dirs
[[ ! -d "$CONFIG_DIR/qwert/zsh" ]] && mkdir -p "$CONFIG_DIR/qwert/zsh"

# copy zsh init to config dir
if [ ! -f $QWERT_ZSH_INIT_CUSTOM ]; then
    cp $QWERT_ZSH_INIT $QWERT_ZSH_INIT_CUSTOM
    echo "  - Create custom zsh init at $QWERT_ZSH_INIT_CUSTOM."
fi
