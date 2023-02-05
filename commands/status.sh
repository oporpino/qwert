#!/bin/bash

subcommand=$1

case $subcommand in
"all")
    $QWERT_DIR/commands/status/all.sh
;;
"homebrew")
    echo "QWERT not implemented this command yet. Fell free to constribute with us."
;;
*)
    echo "I dont understand what you want. Please check the parameters"
;;
esac

