#!/bin/bash

subaction=$1

case $subaction in
"macos")
    $QWERT_DIR/macos/setup/all.sh
;;
*)
    echo "I dont understand what you want."
;;
esac

echo -e "\n[ATTENTION] Run ':PackerSync' when open the Lunar Vim to install packages"
echo -e "[ATTENTION] Run '<prefix>+I' when open the tmux to install plugins"
echo -e "\nEnjoy QWERTT!!!"
