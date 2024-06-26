# .zprofile

# Shell environment

# To see what the system provides, run:
#
#     env -i /bin/bash --norc --noprofile -c 'source /etc/profile; echo $PATH'
#
export PATH=\
~/bin:\
~/.cargo/bin:\
~/.pyenv/shims:\
/usr/local/bin:\
/usr/local/sbin:\
/usr/bin:\
/usr/sbin:\
/bin:\
/sbin

# To see what the system provides, run:
#
#   env -i man --path
#
export MANPATH=\
/usr/local/share/man:\
/usr/share/man:\
/Library/Apple/usr/share/man:\
/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/share/man:\
/Library/Developer/CommandLineTools/usr/share/man

# Typically not set by the system but let's set it
export INFOPATH=\
/usr/local/share/info:\
/usr/share/info

export EDITOR=/usr/local/bin/emacsclient

# This is where we'd set the umask for more privacy. Child processes inherit the umask from parent
# processes, so it is correct to put this in the profile, not the rc. See
# http://en.wikipedia.org/wiki/Umask#Processes.
#
# I've run with a restricted umask for a long time now and while it doesn't really cause any
# problems, it also doesn't seem very useful on a personal macOS machine. Disabling for now but
# leaving it in here for posterity.

#umask u=rwx,g=,o=

# Unrolled from `pyenv init - zsh'
export PYENV_SHELL=zsh
# The output also contains logic to add to the PATH and I've simply added the shims directory to the
# PATH directly where that is set earlier in the file.

# Disable Homebrew auto-update.
#
# The setting in /etc/homebrew/brew.env ought to be enough, but I'm have trouble with that taking
# effect. Set the environment variable too. See #43.
#
# https://docs.brew.sh/Manpage#environment
export HOMEBREW_NO_AUTO_UPDATE=1
