#!/bin/bash

# install tmux
which -s tmux`
if [[ $? != 0 ]] ; then
    echo "  - Start to install TMUX"
    brew install tmux
    echo "  - Start to install TMUX Plugin Manager"
    git clone https://github.com/tmux-plugins/tpm ~/.tmux/plugins/tpm
else
    echo "  - [warn] TMUX is already isntalled. To reinstall please check the TMUX documentation."
fi

