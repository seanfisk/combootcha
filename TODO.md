# Tasks

## Must do

- Port Emacs config
- emacs-mac/cask conflict: https://github.com/d12frosted/homebrew-emacs-plus#known-issues
- Audit chef-repo for stuff to transfer

## Optional

Consider creating GitHub issues for these.

### Prompts

Starship, recommended by Alex

### iTerm2 recent directories integration

Should switch to zoxide or similar since fasd is dead. Whatever you do, rewrite the tool in Rust.

### Screenclean

Set macOS screenshots dir to `~/Pictures/Screenshots`. Sweep the directory daily and use `trash` to remove anything older than 7 days.

Use `trash` for now but note that the way it trashes items is deprecated. Now we should use [`NSFileManager`](https://developer.apple.com/documentation/foundation/nsfilemanager/1414306-trashitematurl). Possibly write a Swift program to do this in the future.

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

### Reduce duplication around writing files

 Lots of `create_file`, `write_all`, then `sync_all`. Could have better context on the errors. Maybe make a helper for it.

### pyenv

It's being installed by Homebrew but we're not configuring it. Definitely need it for work, but we may want to include it for personal too.
