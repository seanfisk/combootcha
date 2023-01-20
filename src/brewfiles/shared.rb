# -*- mode: ruby -*-

# Bundler for Homebrew
# https://github.com/Homebrew/homebrew-bundle

tap 'homebrew/command-not-found'

brew 'bash'
brew 'cask'
brew 'coreutils'
brew 'defaultbrowser'
# Dos2Unix / Unix2Dos <http://waterlan.home.xs4all.nl/dos2unix.html> looks
# superior to Tofrodos <http://www.thefreecountry.com/tofrodos/>. But that was
# just from a quick look.
brew 'dos2unix'
brew 'duti'
brew 'editorconfig'
brew 'exa'
brew 'fasd'
brew 'git'
brew 'git-lfs'
brew 'gnu-tar'
brew 'graphicsmagick'
brew 'hub'
brew 'nmap'
# I prefer ohcount to cloc and sloccount.
brew 'ohcount'
brew 'pandoc'
brew 'pyenv'
brew 'pyenv-virtualenv'
brew 'reattach-to-user-namespace'
brew 'renameutils'
brew 'ripgrep'
brew 'rustup-init'
brew 'ssh-copy-id'
brew 'trash'
brew 'watch'
brew 'wget'
brew 'xz'
brew 'zsh'

tap 'railwaycat/emacsmacport'
brew 'emacs-mac', args: ['with-spacemacs-icon']

cask 'atext'
cask 'caffeine'
# TODO Cathode
cask 'dash'
cask 'deeper'
cask 'disk-inventory-x'
cask 'flux'
cask 'gimp'
cask 'hammerspoon'
cask 'iterm2'
cask 'jettison'
cask 'karabiner-elements'
cask 'lastpass'
cask 'speedcrunch'
cask 'wireshark'
cask 'inkscape' # No longer needs XQuartz

tap 'homebrew/cask-fonts'
cask 'font-inconsolata'
cask 'font-ubuntu'

mas 'Microsoft To Do', id: 1274495053
