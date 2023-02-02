DEFAULT_GOAL := help

.PHONY: test install

TMUX_FILE=.tmux.conf
TMUX_FILE_BKP=$(TMUX_FILE).bkp

help:
	@echo “help needed”

# tmux section
tmux-setup: tmux-backup tmux-link

tmux-backup:
	test -f ~/$(TMUX_FILE) && mv ~/$(TMUX_FILE) ~/$(TMUX_FILE_BKP)

tmux-link:
	ln -s ~/.porpino/config/tmux/$(TMUX_FILE) ~/$(TMUX_FILE)

tmux-rollback:
	test -f ~/$(TMUX_FILE) && rm -f ~/$(TMUX_FILE)
	test -f ~/$(TMUX_FILE_BKP) && mv -f ~/$(TMUX_FILE_BKP) ~/$(TMUX_FILE)

# action section
install: tmux-setup

unistall: tmux-rollback

