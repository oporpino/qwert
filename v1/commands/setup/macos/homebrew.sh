#!/bin/bash

# install homebrew
which -s brew
if [[ $? != 0 ]] ; then
    echo "  - Start to install homebrew"
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
else
    echo "  - [warn] Homebrew is already installed. To reinstall please check the Homebrew documentation. You also can run 'brew update' if is needed."
fi

echo "  - Instaling Homebrew plugins"
which -s sed
if [[ $? != 0 ]] ; then
    echo "    + Instaling gnu-sed"
    brew install gnu-sed
else
    echo "    + [warn] gnu-sed is already installed"
fi

which -s complete
if [[ $? != 0 ]] ; then
    echo "    + Instaling bash_completion"
    brew install bash-completion
else
    echo "    + [warn] bash_completion is already installed"
fi
