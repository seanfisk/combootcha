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
brew 'emacs-mac', args: ['with-spacemacs-icon']
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

cask 'atext'
cask 'caffeine'
# TODO Cathode
cask 'dash'
cask 'deeper'
cask 'disk-inventory-x'
cask 'flux'
cask 'font-inconsolata'
cask 'font-ubuntu'
cask 'gimp'
cask 'hammerspoon'
cask 'iterm2'
cask 'jettison'
cask 'karabiner-elements'
cask 'lastpass'
cask 'speedcrunch'
cask 'wireshark'

# X11-based software
# Note: XQuartz is installed to /Applications/Utilities/XQuartz.app
#cask 'xquartz'
# These formulae require XQuartz to be installed first
#brew 'xclip'
# Let's try it without XQuartz first and see what happens
cask 'inkscape'
