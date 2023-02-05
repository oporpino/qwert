#!/bin/bash

# constants
QWERT_USER_CONFIG_DIR=$HOME/.config/qwert
QWERT_DEFAULTS_CONFIG_DIR=$QWERT_DIR/shared/config

# ZSH ------------------------------------------------------------------------------------
# creating config dirs
[[ ! -d "$QWERT_USER_CONFIG_DIR/zsh" ]] && mkdir -p "$QWERT_USER_CONFIG_DIR/zsh"

# copy zsh init to config dir
if [ ! -f "$QWERT_USER_CONFIG_DIR/zsh/init.sh" ]; then
    echo "  - Create custom zsh init at $QWERT_USER_CONFIG_DIR."
    cp $QWERT_DEFAULTS_CONFIG_DIR/zsh/init.sh $QWERT_USER_CONFIG_DIR/zsh/init.sh
fi

# TMUX -----------------------------------------------------------------------------------
# creating config dirs
[[ ! -d "$QWERT_USER_CONFIG_DIR/tmux" ]] && mkdir -p "$QWERT_USER_CONFIG_DIR/tmux"

# copy and link tmux.conf to config dir
if [ ! -f "$QWERT_USER_CONFIG_DIR/tmux/tmux.conf" ]; then
    echo "  - Create custom tmux.conf at $QWERT_USER_CONFIG_DIR."
    cp $QWERT_DEFAULTS_CONFIG_DIR/tmux/tmux.conf $QWERT_USER_CONFIG_DIR/tmux/tmux.conf

    echo "  - Backup and link custom tmux.conf"
    mv $HOME/.tmux.conf $HOME/.tmux.conf.bkp
    ln -s $QWERT_DEFAULTS_CONFIG_DIR/tmux/tmux.conf $HOME/.tmux.conf
fi

# LVIM -----------------------------------------------------------------------------------
# creating config dirs
[[ ! -d "$QWERT_USER_CONFIG_DIR/lvim" ]] && mkdir -p "$QWERT_USER_CONFIG_DIR/lvim"

# copy and link lvim default to config dir
if [ ! -f "$QWERT_USER_CONFIG_DIR/lvim/config.lua" ]; then
    echo "  - Create custom lvim/config.lua at $QWERT_USER_CONFIG_DIR."
    cp $QWERT_DEFAULTS_CONFIG_DIR/lvim/config.lua $QWERT_USER_CONFIG_DIR/lvim/config.lua

    echo "  - Backup and link custom config.lua"
    mv $HOME/.config/lvim/config.lua $HOME/.config/lvim/config.lua.bkp
    ln -s $QWERT_DEFAULTS_CONFIG_DIR/lvim/config.lua $HOME/.config/lvim/config.lua
fi
