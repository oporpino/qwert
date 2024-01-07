#!/bin/bash

# install noe vim
which -s neovim
if [[ $? != 0 ]] ; then
    echo "  - Start to install NeoVim"

    brew install neovim 
else
    echo "  - [warn] NeoVim is already installed. To reinstall please check the NeoVim documentation."
fi


