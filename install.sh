#!/bin/bash

ZSH_FILE=$HOME/.zshrc
OH_MY_ZSH_DIR=$HOME/.oh-my-zsh
QWERT_INIT_FILE='"$HOME/.qwert/init.sh"'

echo '> Installing dependencies:'
if [ ! -d $OH_MY_ZSH_DIR ]
then
    echo '  - Instaling Oh-My-Zsh'
    sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"
else
    echo "  - [warn] Oh-My-Zsh is already installed. If you need to reinstall please remove $OH_MY_ZSH_DIR, or check Oh-My-Zsh documentation."
fi

echo '> Configuring initialization:'
if ! grep -q $QWERT_INIT_FILE "$ZSH_FILE"; then
    echo "  - Add QWERT to $ZSH_FILE"
    echo ". $QWERT_INIT_FILE" >> $ZSH_FILE
else
    echo "  - [warn] QWERT is already configured into $ZSH_FILE. No action needed."
fi
