#!/bin/bash

# install homebrew
which -s brew
if [[ $? != 0 ]] ; then
    echo "  - Start to install homebrew"
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
else
    echo "  - [warn] Homebrew is already isntalled. To reinstall please check the Homebrew documentation. You also can run 'brew update' if is needed."
fi
