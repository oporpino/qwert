#!/usr/bin/zsh

export QWERT_DIR=$HOME/.qwert
QWERT_CONFIG_DIR=$HOME/.config/qwert
QWERT_CUSTOM_ZSH_FILE=$HOME/.config/qwert/zsh/init.sh

# Add qwert to PATH
#
# if in $PATH, remove, regardless of if it is in the right place (at the front) or not.
# replace all occurrences - ${parameter//pattern/string}
QWERT_BIN="${QWERT_DIR}/bin"
[[ ":$PATH:" == *":${QWERT_BIN}:"* ]] && PATH="${PATH//$QWERT_BIN:/}"

# add to front of $PATH
PATH="${QWERT_BIN}:$PATH"

if [ -f $QWERT_CUSTOM_ZSH_FILE ]; then
    . $QWERT_CUSTOM_ZSH_FILE
fi
