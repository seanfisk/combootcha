# Tasks

## Must do

- Zsh basics including aliases
  - `alias e='/usr/local/bin/emacsclient --no-wait'`
  - Set `EDITOR` to emacsclient
  - umask: Include this but have it commented out. It's not necessary on a macOS personal system.
  - PATH & friends
- Port Emacs config
- emacs-mac/cask conflict: https://github.com/d12frosted/homebrew-emacs-plus#known-issues
- Audit chef-repo for stuff to transfer

## Optional

Consider creating GitHub issues for these.

### Prompts

Starship, recommended by Alex

### iTerm2 recent directories integration

Should switch to zoxide or similar since fasd is dead. Whatever you do, rewrite the tool in Rust.

### Zsh plugin manager options

1. https://github.com/zdharma/zinit
1. https://github.com/zplug/zplug
1. https://github.com/rossmacarthur/sheldon
1. http://getantibody.github.io/

https://github.com/vintersnow/zsh_plugin_manager_speed
https://www.reddit.com/r/zsh/comments/ak0vgi/a_comparison_of_all_the_zsh_plugin_mangers_i_used/
https://jdhao.github.io/2019/10/08/zsh_plugin_managers_compare/

### Configuration for Bash as well

I don't use Bash much so this seems unnecessary.

### Homebrew API token

Do we still need this?

### iTerm2 shell integration

Need to investigate how this is done now. My stuff in dotfiles was written a long time ago.

### Old

Instead of just `old` and `unold`, write a utility that does better handling and allows accepting the filename without the `.old` suffix

### gcm

Modify this to work with master or main

### Capture

`capture` script and `ccapture` alias
