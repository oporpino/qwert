#!/bin/bash

ZSH_FILE=$HOME/.zshrc
OH_MY_ZSH_DIR=$HOME/.oh-my-zsh
QWERT_DIR=$HOME/.qwert
QWERT_INIT_FILE='"$HOME/.qwert/init.sh"'
QWERT_INIT_COMPLETIONS_FILE='"$HOME/.qwert/scripts/completions.sh"'
QWERT_RELOAD_CMD='reload!="source ~/.zshrc"'

echo '> Installing dependencies:'
if [ ! -d $OH_MY_ZSH_DIR ]
then
    echo '  - Instaling Oh-My-Zsh'
    sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"
else
    echo "  - [warn] Oh-My-Zsh is already installed. If you need to reinstall please remove $OH_MY_ZSH_DIR, or check Oh-My-Zsh documentation."
fi

echo '> Setup QWERT:'
if [ ! -d $QWERT_DIR ]
then
    echo '  - Download QWERT source code'
    git clone https://github.com/gporpino/qwert.git $QWERT_DIR
    echo '  - Configuring QWERT'
    rm -rf $QWERT_DIR/.git
else
    echo "  - [warn] QWERT is already installed. If you need to reinstall please remove $QWERT_DIR, or check QWERT documentation."
fi

echo '> Configuring QWERT initialization:'
if ! grep -q $QWERT_INIT_FILE "$ZSH_FILE"; then
    echo "  - Add QWERT to $ZSH_FILE"
    echo ". $QWERT_INIT_FILE" | cat - $ZSH_FILE > temp && mv temp $ZSH_FILE
else
    echo "  - [warn] QWERT is already configured into $ZSH_FILE. No action needed."
fi

echo '> Configuring reload alias command:'
if ! grep -q $QWERT_RELOAD_CMD "$ZSH_FILE"; then
    echo "  - Add reload alias to $ZSH_FILE"
    echo "alias $QWERT_RELOAD_CMD" >> $ZSH_FILE
else
    echo "  - [warn] reload alias is already configured into $ZSH_FILE. No action needed."
fi

echo '> Configuring initialization completions:'
if ! grep -q $QWERT_INIT_COMPLETIONS_FILE "$ZSH_FILE"; then
    echo "  - Add QWERT completions to $ZSH_FILE"
    echo ". $QWERT_INIT_COMPLETIONS_FILE" >> $ZSH_FILE
else
    echo "  - [warn] QWERT completions is already configured into $ZSH_FILE. No action needed."
fi

echo '> QWERT was sucessful installed'

unset ZSH_FILE OH_MY_ZSH_DIR QWERT_INIT_FILE
