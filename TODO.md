# Tasks

- Firefox remember tabs on work machine
- Start at login on work machine
  - Firefox
  - Outlook
  - Slack
- Add homebrew tvs stuff
  - All formulae
  - .bm_pref and stuff from developer guide
- Shell setup
- Zip up Cathode.app from my current computer and include it and the license directly in the app
- Find the license at and install it directly to `~/Library/Application Support/Cathode/License.cathodelicense`
- fasd/iterm2 integration
- Use Hammerspoon instead of Quicksilver to activate shortcuts for iTerm and Firefox
- emacs-mac/cask conflict: https://github.com/d12frosted/homebrew-emacs-plus#known-issues
- Upgrade to Clap 4.x
- In battery, set *Wake for network access* to *Never*


Install Cathode

Made an archive of the app with this command:

      tar --uid 0 --uname root --gid 0 --gname root -cJvf Cathode-2.4.1.tar.xz -C /Applications Cathode.app

https://apple.stackexchange.com/questions/144656/can-i-make-the-trackpad-tracking-speed-faster-than-allowed-in-system-preferences

https://gist.github.com/brandonb927/3195465

https://github.com/mathiasbynens/dotfiles/blob/master/.macos

# Things to add

When there are several options to try, they're ordered by preference from cursory research.

## Zsh plugin manager options

1. https://github.com/zdharma/zinit
1. https://github.com/zplug/zplug
1. https://github.com/rossmacarthur/sheldon
1. http://getantibody.github.io/

https://github.com/vintersnow/zsh_plugin_manager_speed
https://www.reddit.com/r/zsh/comments/ak0vgi/a_comparison_of_all_the_zsh_plugin_mangers_i_used/
https://jdhao.github.io/2019/10/08/zsh_plugin_managers_compare/

## Prompts

Starship, recommended by Alex

## Port relevant sections from dotfiles

- Capture
- Shell configurations

## Critical utilities

- `dns`
- `e`
- `rdns`
- `unattend`

## Port Emacs config

## macOS do not wake on enhanced notifications

https://osxdaily.com/2017/12/20/stop-enhanced-notifications-waking-mac-sleep/
