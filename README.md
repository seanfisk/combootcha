# Combootcha

A set of programs to bootstrap my personal computing setup.

I had previously used these three repos to automate my setup, which use the following tools:

- https://github.com/seanfisk/personal-chef-repo (Chef)
- https://github.com/seanfisk/dotfiles (Waf)
- https://github.com/seanfisk/emacs (Waf)

I have learned several lessons from this:

- Chef is a massive amount of complexity. It might be worthwhile for DevOps, but when you are only managing several personal machines, a scripted approach is sufficient.
- Waf requires a Python development environment and doesn't work perfectly for something like configuring and installing a personal setup.

At my core, I am a programmer, and I want everything the way I want it. Instead of using prescriptive tools like Chef and Waf, I think it will be simpler to have a program that does everything that is necessary *exactly* as I've prescribed, and nothing more.

This also removes the decision of where to put customizations. The emacs repo was fairly obvious, but the distinction between personal-chef-repo and dotfiles grew increasingly unclear.

A program will be built for each environment. Currently planned:

- Work laptop
- Personal laptop

## Executing the setup

### First time

Before doing anything, it is important to set up Time Machine. Select a free partition on an external hard drive and add it to Time Machine. I don't think the partition needs to be erased and reformatted — I think Time Machine will do this automatically. Add the disk and run a backup before going farther.

Next, install either Xcode or Command Line Tools. For Homebrew, Python, Ruby, and normal C++ compiles, we can get along with just the Command Line Tools. However, to compile Mac applications using Qt, we need the full Xcode installation.

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
sudo ./combootcha --homebrew --set-default-browser work
```

### Subsequent updates

This is easy:

```bash
sudo ./combootcha --homebrew work
```

If you don't feel like updating Homebrew, that can of course be omitted.

## References on automating macOS

- https://github.com/tiiiecherle/osx_install_config
