#!/bin/bash

if [[ $OSTYPE == 'darwin'* ]]; then
    which -s brew
    if [[ $? == 0 ]] ; then
        echo '  - [Instaled] Homebrew'
    else
        echo '  - [Not Instaled] Homebrew'
    fi
fi
