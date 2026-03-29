#!/bin/bash

# install git
which -s git
if [[ $? != 0 ]] ; then
    echo "  - Start to install Git"
    brew install git
else
    echo "  - [warn] Git is already installed. To reinstall please check the Git documentation."
fi

# install delta (git diff pager)
which -s delta
if [[ $? != 0 ]] ; then
    echo "  - Start to install Delta"
    brew install git-delta
else
    echo "  - [warn] Delta is already installed. To reinstall please check the Delta documentation."
fi

# configure git to use delta
echo "  - Configuring git to use Delta"
git config --global core.pager delta
git config --global interactive.diffFilter "delta --color-only"
git config --global delta.navigate true
git config --global delta.light false
git config --global delta.side-by-side true
git config --global merge.conflictstyle diff3
git config --global diff.colorMoved default

echo "  - Delta configured successfully"
