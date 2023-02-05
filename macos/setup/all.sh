#!/bin/bash

echo "> Initializing setup enviroment for macos"

. $QWERT_DIR/macos/setup/homebrew.sh
. $QWERT_DIR/macos/setup/lvim.sh
. $QWERT_DIR/macos/setup/tmux.sh
. $QWERT_DIR/macos/setup/config.sh

echo "> Setup os macos enviroment finished."

echo -e "\n[ATTENTION] Run ':PackerSync' when open the Lunar Vim to install packages"
echo -e "[ATTENTION] Run '<prefix>+I' when open the tmux to install plugins"
echo -e "\nEnjoy QWERTT!!!"

