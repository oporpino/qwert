#!/bin/bash

# install tmux
which -s wemux
if [[ $? != 0 ]] ; then
    echo "  - Start to install WEMUX"
    brew install wemux
    echo "  - Start to install WEMUX Plugin Manager"
else
    echo "  - [warn] WEMUX is already installed. To reinstall please check the WEMUX documentation."
fi

