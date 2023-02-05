#!/bin/bash

subaction=$1

case $subaction in
"macos")
    $QWERT_DIR/macos/setup/all.sh
;;
"linux")
    echo "QWERT for linux is not implemented yet. Fell free to constribute with us."
;;
*)
    echo "I dont understand what you want. Please check the parameters"
;;
esac

