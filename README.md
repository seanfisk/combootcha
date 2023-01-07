# Sean Fisk's Computing Setup

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
- Personal laptop

## Executing the setup

First, install either Xcode or Command Line Tools. For Homebrew, Python, Ruby, and normal C++ compiles, we can get along with just the Command Line Tools. However, to compile Mac applications using Qt, we need the full Xcode installation.

If using Command Line Tools, run `xcode-select --install`. If using Xcode, open the Mac App Store and install Xcode from there.

Next, change some permissions that Homebrew needs to install Zsh completions. When I ran this setup in early 2023, I had to run this manually to get Homebrew to install properly:

```bash
sudo chown -R $(whoami) /usr/local/share/zsh
chmod u+w /usr/local/share/zsh
```

Next, install Homebrew manually by [following the instructions](https://brew.sh/#install). I am having trouble automating Homebrew system install in this current setup and it doesn't seem worth fretting over automating it right now since it's a one-time install.

After this, run `brew doctor` and attempt to address any problems that are reported.

Next, transfer the compiled executable to the new system and execute it:

```bash
sudo ./computing-setup --homebrew --set-default-browser work
```
