# Sean Fisk's Personal Computing Setup

A set of programs to automate my personal computing setup.

I had previously used these three repos to automate my setup, which use the following tools:

- https://github.com/seanfisk/personal-chef-repo (Chef)
- https://github.com/seanfisk/dotfiles (Waf)
- https://github.com/seanfisk/emacs (Waf)

I have learned several lessons from this:

- Chef is a massive amount of complexity. It might be worthwhile for DevOps, but when you are only managing several personal machines, a scripted approach is sufficient.
- Waf requires a Python development environment and doesn't work perfectly for something like configuring and installing a personal setup.

At my core, I am a programmer, and I like everything just so. Instead of using prescriptive tools like Chef and Waf, I think it will be simpler to have a program that does everything that is necessary in *exactly* the way I want it, and nothing more.

This also removes the decision of where to put customizations. The emacs repo was fairly obvious, but the distinction between personal-chef-repo and dotfiles grew increasingly unclear.

A program will be built for each environment. Currently planned:

- Work laptop
- vROps boxes (mostly just to have a second environment, which forces me to consider it)

## Executing the setup

TODO
