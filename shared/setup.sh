#!/bin/bash

subaction=$1

case $subaction in
"macos")
    $QWERT_DIR/macos/setup.sh $subaction
;;
*)
    echo "foo or bar was not sent"
;;
esac
