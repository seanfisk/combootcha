# Tasks

## Must do

- Port Emacs config
- emacs-mac/cask conflict: https://github.com/d12frosted/homebrew-emacs-plus#known-issues

## Optional

Consider creating GitHub issues for these.

### Screenclean

Set macOS screenshots dir to `~/Pictures/Screenshots`. Sweep the directory daily and use `trash` to remove anything older than 7 days.

Use `trash` for now but note that the way it trashes items is deprecated. Now we should use [`NSFileManager`](https://developer.apple.com/documentation/foundation/nsfilemanager/1414306-trashitematurl). Possibly write a Swift program to do this in the future.

### Capture

`capture` script and `ccapture` alias

### Reduce duplication around writing files

Lots of `create_file`, `write_all`, then `sync_all`. Could have better context on the errors. Maybe make a helper for it.

### pyenv

It's being installed by Homebrew but we're not configuring it. Definitely need it for work, but we may want to include it for personal too.
