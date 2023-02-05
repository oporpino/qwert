#!/bin/bash

# install lunar vim
which -s lvim
if [[ $? != 0 ]] ; then
    echo "  - Start to install Lunar Vim"
    LV_BRANCH='release-1.2/neovim-0.8' bash <(curl -s https://raw.githubusercontent.com/lunarvim/lunarvim/fc6873809934917b470bff1b072171879899a36b/utils/installer/install.sh)
else
    echo "  - [warn] LunarVim is already isntalled. To reinstall please check the LunarVim documentation."
fi

