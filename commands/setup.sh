#!/bin/bash

subcommand=$1

case $subcommand in
"macos")
    $QWERT_DIR/commands/setup/macos/all.sh
;;
"linux")
    echo "QWERT for linux is not implemented yet. Fell free to constribute with us."
;;
*)
    echo "I dont understand what you want. Please check the parameters"
;;
esac

