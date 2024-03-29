# Combootcha

A set of programs to bootstrap my personal computing setup.

I had previously used these three repos to automate my setup, which use the following tools:

- https://github.com/seanfisk/personal-chef-repo (Chef)
- https://github.com/seanfisk/dotfiles (Waf)
- https://github.com/seanfisk/emacs (Waf)

I have learned several lessons from this:

- Chef is a massive amount of complexity. It might be worthwhile for DevOps, but when you are only managing several personal machines, a scripted approach is sufficient.
- Waf requires a Python development environment and doesn't work perfectly for something like configuring and installing a personal setup.

At my core, I am a programmer, and I want everything the way I want it. Instead of using prescriptive tools like Chef and Waf, I think it will be simpler to have a program that does everything that is necessary *exactly* as I've instructed, and nothing more.

This also removes the decision of where to put customizations. The emacs repo was fairly obvious, but the distinction between personal-chef-repo and dotfiles grew increasingly unclear.

A program has been built for each environment, currently:

- Personal laptop (this repository)
- Work laptop (private repository stored at work)

## References on automating macOS

- https://github.com/tiiiecherle/osx_install_config
- https://github.com/mathiasbynens/dotfiles/blob/master/.macos
- https://gist.github.com/brandonb927/3195465
- Good technique for determining what preferences changed: https://apple.stackexchange.com/a/457024
