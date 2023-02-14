# QWERT
### The dev tools manager!

Setup your dev enviroment with all your tools in minutes and just once!

# Install
Install via `curl`. 

```
sh -c "$(curl -fsSL  https://raw.githubusercontent.com/gporpino/qwert/v0.1.1/scripts/install.sh)"
```


# Usage

## macOS
run to setup enviroment for macOS:

```
qwert setup macos
```

## linux
\# Not implemented yet.


## Avaliable Tools

- Homebrew
- LunarVim
- Tmux

### TMUX:
- Plugin Manager Installed with some plugins by default. Run `<prefix> + I` to install plugins.
- Bind some intuitive shortcuts.

#### Shortcuts
- \<prefix>+"-": To slip window horizontaly
- \<prefix>+"|": To slip window verticaly
- \<prefix>+"r": To reload configurations.

See all custom shortcuts at `~/.tmux.conf`

### Lunar Vim
- Some optional plugins installed

### Homebrew
- Install `Homebrew` and some plugins

## Customization
We also made some custom mods to improve the usage of tools, but you can extend and change as you want.

To save your customization you just need to save `.config/qwert` folder, and all your enviroment will be the same. We recomend to you save the `.config` in a private repository. 

# Dependencies
The qwert needs `curl`, `oh-my-zsh` to works properly. Don`t worry, we will install it on QWERT instalation.

# Coming Soon
We will implements many others capabilities. 

Roadmap tools
- ASDF
- PowerShell 10k
- Postgree

Please, contribute with us sending PRs! 🙏
