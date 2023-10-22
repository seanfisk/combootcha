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

# Aliases
alias c='cd'
alias fsb='stat -f %z'
alias godmode='sudo -i'
alias less='less -R'
alias lslisten='/usr/sbin/lsof -nP -iTCP -sTCP:LISTEN'
alias mkdate='date +%Y-%m-%d'
alias py=python
alias rp=/usr/local/bin/grealpath
alias tcping='/usr/local/bin/ncat --verbose -z --wait 1'
alias u='cd ..' # thanks Karlin

# Clipboard
alias ccopy=/usr/bin/pbcopy
alias cpaste=/usr/bin/pbpaste
alias ccwd='echo -n "$(pwd)" | ccopy'

# Git
alias g=/usr/local/bin/hub
alias gcm='/usr/local/bin/hub checkout master'
alias git=/usr/local/bin/hub
alias gobuddygo='/usr/local/bin/hub push'
alias gt='/usr/local/bin/hub status'

# Directory listing
alias l='ls --long --git'
alias la='l --all'
alias ls='/usr/local/bin/eza --classify'

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
