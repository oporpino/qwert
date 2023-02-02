DEFAULT_GOAL := help

.PHONY: test install

# contants
CONFIG_HOME=$(HOME)/.porpino/config
ZSH_FILE=.zshrc
TMUX_FILE=.tmux.conf
LVIM_FILE=config.lua
LVIM_HOME=$(HOME)/.config/lvim

help:
	@echo “help needed”

# zsh section
zsh-setup:
	chmod +x $(CONFIG_HOME)/zsh/init.sh
	echo ./$(CONFIG_HOME)/zsh/init.sh >> $(HOME)/$(ZSH_FILE)

# [ TMUX SECTION ]
tmux-setup: tmux-backup tmux-setup-plugin-manager tmux-link

tmux-backup:
	test -f $(HOME)/$(TMUX_FILE) && mv $(HOME)/$(TMUX_FILE) $(HOME)/$(TMUX_FILE).bkp

tmux-link:
	ln -s $(CONFIG_HOME)/tmux/$(TMUX_FILE) $(HOME)/$(TMUX_FILE)

tmux-setup-plugin-manager:
	git clone https://github.com/tmux-plugins/tpm ~/.tmux/plugins/tpm

tmux-rollback:
	test -f $(HOME)/$(TMUX_FILE) && rm -f $(HOME)/$(TMUX_FILE)
	test -f $(HOME)/$(TMUX_FILE).bkp && mv -f $(HOME)/$(TMUX_FILE).bkp $(HOME)/$(TMUX_FILE)

# [ LVIM SECTION ]
lvim-setup: lvim-backup lvim-link

lvim-backup:
	test -f $(LVIM_HOME)/$(LVIM_FILE) && mv $(LVIM_HOME)/$(LVIM_FILE) $(LVIM_HOME)/$(LVIM_FILE).bkp

lvim-link:
	ln -s $(CONFIG_HOME)/lvim/$(LVIM_FILE) $(LVIM_HOME)/$(LVIM_FILE)

lvim-rollback:
	test -f $(LVIM_HOME)/$(LVIM_FILE) && rm -f $(LVIM_HOME)/$(LVIM_FILE)
	test -f $(LVIM_HOME)/$(LVIM_FILE).bkp && mv -f $(LVIM_HOME)/$(LVIM_FILE).bkp $(LVIM_HOME)/$(LVIM_FILE)

# action section
install: tmux-setup lvim-setup

unistall: tmux-rollback lvim-rollback


