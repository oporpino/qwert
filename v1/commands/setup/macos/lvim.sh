#!/bin/bash

# install lunar vim
which -s lvim
if [[ $? != 0 ]] ; then
    echo "  - Start to install Lunar Vim"

    LV_BRANCH='release-1.3/neovim-0.9' bash <(curl -s https://raw.githubusercontent.com/LunarVim/LunarVim/release-1.3/neovim-0.9/utils/installer/install.sh)
else
    echo "  - [warn] LunarVim is already installed. To reinstall please check the LunarVim documentation."
fi


