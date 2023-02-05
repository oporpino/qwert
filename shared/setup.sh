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
