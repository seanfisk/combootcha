# .zshrc

# Exit if non-interactive. There are many reasons to do this:
# - Aliases, etc. are not typically necessary for non-interactive scripts
# - Speeds up execution time for non-interactive subshells
# - We need to "keep the shell clean." See `man rsync` for more info on that.
[[ $- != *i* ]] && return

# Version check
MIN_ZSH_VERSION=5
if [[ $ZSH_VERSION[0,1] -lt $MIN_ZSH_VERSION ]]; then
  echo >&2 "This configuration is compatible only with Zsh version $MIN_ZSH_VERSION and upwards. Please update your Zsh version."
  return
fi
unset MIN_ZSH_VERSION

# Shell options
setopt INTERACTIVE_COMMENTS

# Files & directories
alias c='cd'
alias u='cd ..' # thanks Karlin
cdl() { cd "$1" && ls; }
mk() { mkdir -p "$1" && cd "$1"; }

alias l='ls --long'
alias la='l --all'
alias ls='eza --classify'

old() { mv "$1" "$1.old"; } # make a *.old file
unold() { mv "$1" "${1%.old}"; }

# Clipboard
alias ccopy=pbcopy
alias cpaste=pbpaste
alias ccwd='echo -n "$(pwd)" | ccopy'

# Git
alias g=hub
alias git=hub
alias gcm='hub checkout master'
alias gobuddygo='hub push'
alias gt='hub status'

# Miscellaneous
alias e='emacsclient --no-wait'
alias fsb='stat -f %z'
alias godmode='sudo -i'
alias less='less -R'
alias lslisten='lsof -nP -iTCP -sTCP:LISTEN'
alias mkdate='date +%Y-%m-%d'
alias py=python
alias rp=grealpath
alias tcping='ncat --verbose -z --wait 1'

# Process-related utilities
#
# All these programs support a -u argument specifying the user. For ps, pgrep, and pkill it is
# effective user id (euid). For htop and lsof this is unspecified. In most of my cases, euid and
# ruid will be the same anyway.
#
# There are two different versions of pstree:
# - http://freecode.com/projects/pstree, used on macOS
# - http://psmisc.sourceforge.net/, used on most GNU/Linux machines
# But they both support the -u flag!
#
# Note: `id -un' was used since `whoami' has been obsoleted and is not POSIX.
for prog in ps pgrep pkill lsof pstree; do
  alias my"$prog"="$prog -u \"$(id -un)\""
done

non-native-dns() {
  echo "The \`$1' command does not use native macOS DNS resolution facilities. Using the \`dns' or \`rdns' alias is recommended." >&2
  $@
}
alias nslookup='non-native-dns nslookup'
alias host='non-native-dns host'

# Paging
#
# Note: `|&' is Bash 4 and Zsh only.
#
# Note: We used to bind this to C-j, but that interferes with tmuxifier's `run_cmd "clear"' line in
# `lib/layout-helpers.sh' for some reason. It causes the command strings to be piped to less, which
# brings the terminal into less and doesn't execute the commands. Not sure about the cause.
bindkey -s '\C-x\C-l' ' |& less\C-m'

# Execute last command. This is equivalent to pressing C-p or the up arrow, then Enter.
bindkey -s '\C-xp' '\C-p\C-m'

# Up a directory, aliased to `u' for me. Note: `\ej' means `ESC+' then `j' as opposed to `\M-j',
# which means `Meta' then `j'. I have both Option keys on my Mac configured to send `ESC+' in
# iTerm2. Actually sending Meta is apparently a relic of the past, and ESC+ should be used now.
bindkey -s '\ej' 'u\C-m'

# Unrolled from `pyenv init - zsh'
source /usr/local/opt/pyenv/completions/pyenv.zsh
command pyenv rehash 2>/dev/null
pyenv() {
  local command
  command="${1:-}"
  if [ "$#" -gt 0 ]; then
    shift
  fi

  case "$command" in
    activate|deactivate|rehash|shell)
      eval "$(pyenv "sh-$command" "$@")"
      ;;
    *)
      command pyenv "$command" "$@"
      ;;
  esac
}
