#!/bin/bash

echo "> Initializing setup enviroment for macos"

. $QWERT_DIR/commands/setup/macos/homebrew.sh
. $QWERT_DIR/commands/setup/macos/neovim.sh
. $QWERT_DIR/commands/setup/macos/lvim.sh
. $QWERT_DIR/commands/setup/macos/tmux.sh
. $QWERT_DIR/commands/setup/macos/wemux.sh
. $QWERT_DIR/commands/setup/macos/config.sh

echo "> Setup os macos enviroment finished."

echo -e "\n[ATTENTION] Run ':PackerSync' when open the Lunar Vim to install packages"
echo -e "[ATTENTION] Run '<prefix>+I' when open the tmux to install plugins"
echo -e "\nEnjoy QWERTT!!!"

