# Executing the setup

## Subsequent updates

This is easy:

```bash
sudo ./combootcha --homebrew
```

If you don't feel like updating Homebrew, that can of course be omitted.

## First time

### Shared

Everything in this section should be done for all setups.

#### Time Machine

Before doing anything, it is important to set up Time Machine. Select a free partition on an external hard drive and add it to Time Machine. I don't think the partition needs to be erased and reformatted — I think Time Machine will do this automatically. Add the disk and run a backup before going farther.

#### Trackpad & keyboard

Time Machine is the top priority, but basic input is the next task. Theoretically, these settings are stored in macOS user defaults and are easy to automate, but in practice I have had difficulty getting them applied through user defaults, even after a restart. Thus, for this section, simply open System Preferences and set as shown in these screenshots.

<img src="screenshots/keyboard-settings.png" alt="Keyboard settings" width="400" />

<img src="screenshots/trackpad-settings/point-and-click.png" alt="Trackpad Point & Click" width="400" />

<img src="screenshots/trackpad-settings/scroll-and-zoom.png" alt="Trackpad Scroll & Zoom" width="400" />

<img src="screenshots/trackpad-settings/more-gestures.png" alt="Trackpad More Gestures" width="400" />

#### Xcode / Command Line Tools

Next, install either Xcode or Command Line Tools. For Homebrew, Python, Ruby, and normal C++ compiles, we can get along with just the Command Line Tools. However, to compile Mac applications using Qt, we need the full Xcode installation.

If using Command Line Tools, run `xcode-select --install`. If using Xcode, open the Mac App Store and install Xcode from there.

#### Homebrew

Next, install Homebrew manually by [following the instructions](https://brew.sh/#install). I have always used the scripted install and not the package installer, but the package installer might be worth a try.

I am having trouble automating Homebrew system install in this current setup and it doesn't seem worth fretting over automating it right now since it's a one-time install.

After this, run `brew doctor` and attempt to address any problems that are reported.

#### SSH key generation

Run `ssh-keygen` to generate a new key. Accept the defaults. Security of this will be improved in the future; see #3.

#### Combootcha

Start by transferring the compiled executable from the old macOS computer to the new one. The program cannot be run immediately. Since the executable is unsigned, the new OS will reject the program as coming from an unidentified developer.

To get the OS to trust the program, open Finder, right-click `combootcha`, and choose *Open*. The program will terminate immediately as it needs to be run as root, but this will allow the program to be run from Terminal.

Next, open Terminal and execute the program as shown below. The directory of execution does not matter, although the home directory is recommended.

Some of the options are disabled by default because they can take a while to run. However, this is the first run, so all options need to be enabled.

```bash
sudo ./combootcha --homebrew --set-default-browser
```

The Homebrew installation process will also trigger the installation of some software from the Mac App Store which requires an Apple ID to be entered. The process is carefully designed such that a password manager will be installed *before* the Apple ID prompt. When prompted for the Apple ID, open the freshly-installed password manager, log in, retrieve the credentials, and enter them into the prompt. The program will then continue!

#### Karabiner

This does not start up properly when using the launch agent approach, so we just use native startup support through the app itself. As of 2023-11-12, Karabiner is automatically added to Login Items.

#### Jettison

This is licensed software that is installed using Homebrew. I purchased a license that has to be manually entered/activated. Don't get confused: I initially purchased Jettison from the Mac App Store, but bought a separate license when I found that the Mac App Store version isn't up-to-date. The license is stored in LastPass.

#### Disk Utility

Open Disk Utility and choose to *Show All Devices*:

![Disk Utility Show All Devices](screenshots/disk-utility-show-all-devices.png)

#### Desktop backgrounds

No desktop background is set automatically, so go set one of my own preference. This might be nice to automate, but I change these from time to time manually, and that would be just one more thing to change every time.

#### Firefox

Disable the auto-redirection of domains. Specifically this is annoying for `localhost`. Hopefully this will be synced. See http://cdivilly.wordpress.com/2013/08/15/disable-firefox-redirecting-to-localhost-com/.

#### Dash

This is licensed software that is installed using Homebrew. Grab the license from LastPass and run `open license.dash-license`.

#### Cathode

This is licensed software bundled with and installed by Combootcha. Copy the license file from LastPass directly to `~/Library/Application Support/Cathode/License.cathodelicense` to license the software.

#### Privacy & Security approvals

At minimum, this software needs to be granted permission:

- Hammerspoon
- Karabiner driver
- Homebrew needs permission for App Management for Cask

Add more software here as I am prompted.

#### Touch ID

Add prints for right pointer, right middle, and left pointer fingers.

#### Startup sound

<!--
When the MacBook Pro boots, it plays the classic startup sound. At a normal volume, the sound is pleasant. However, the sound will be played at the volume level that was set when the MacBook Pro was shut down. While this makes some sense, I never have this in mind when I shut down, and why would I? So the startup sound can often play at a very high volume, which is jarring. Disable it for this reason.

I wasn't able to find user defaults for this setting using the diff approach.
-->

Disable the startup sound by navigating to *System Settings* > *Sound* > *Play sound on startup*. The exact instructions vary by macOS version and are [documented by Apple](https://support.apple.com/en-us/102230).

<!--

TODO Update for new aText
https://github.com/seanfisk/combootcha/issues/15

#### aText

This is licensed software that is installed using Homebrew Cask. I purchased it from the Mac App Store (MAS). However, as stated in [aText Support](http://www.trankynam.com/atext/support.html), the non-MAS version generally works better. You'll have to follow the process to migrate your MAS license to get the non-MAS version working.

When using the launch agent approach to start up aText, it does not validate the license. So we're just going for the regular startup process.

-->

### Personal

Everything in this section is for my personal setup only.

#### GitHub

Upload my SSH public key to my GitHub account so that I can commit code.

#### LastPass

The universal installer has installer and uninstaller app bundles that need to be run manually. Using the universal installer is preferable to individual browser add-ons due to the inclusion of all browser add-ons and the binary component, which allows sharing state between browsers. The universal installer is not present in Homebrew, Homebrew Cask, or the Mac App Store and must be downloaded manually from here: https://lastpass.com/misc_download2.php

#### iCloud

Open System Preferences and either follow the promt or simply find the place to sign in and set up iCloud. Choose to sync the following settings:

- Photos
- Keychain
- iCloud Drive
- Contacts
- Calendars
- Notes
- Safari
- Find My Mac
- Siri

All others should be unchecked.

#### Messages

Set up the following preferences for Messages.app:

![Messages.app settings](screenshots/messages-settings.png)

Next, follow the steps outlined in [this article from Apple](https://support.apple.com/guide/messages/get-sms-texts-from-iphone-on-your-mac-icht8a28bb9a/14.0/mac/14.0) to get SMS working in the app.

#### Firefox

Sign in to Firefox Sync. Choose to sync these items:

- Bookmarks
- History
- Open tabs
- Add-ons
- Settings

All others should be unchecked.

Install the following add-ons:

- Bitwarden Password Manager
- uBlock Origin

#### Set default email reader

The Mail app will be the default email reader if none is set. I prefer to use native Gmail. As of 2024-06-13, setting a Safari web app for Gmail as the default email reader does not function properly. So I think it's best to set Firefox as the default email reader, which works fine in the uncommon case that I actually open a `mailto` link.

To configure this, first open the Mail app. Awkwardly, an account must be configured in order to access the settings. Just configure iCloud mail. I don't use it anyway and it's fine to leave configured. Then go to *Mail* > *Settings* > *General* > *Default email reader* and choose Firefox. Now test it by clicking [this `mailto` link](mailto:sean@seanfisk.com?subject=Default%20email%20reader%20test&body=Do%20not%20send%20this%20email.). Choose Gmail and check the box to *Always use this application to open mailto links*.

<!-- Source: https://support.apple.com/en-us/102362 -->

#### Microsoft To Do

Sign in and ensure that all todos have synced properly.

#### Slack

Open the app and sign into both [Blue Medora Bros](https://bluemedorabros.slack.com/) and [Grand Rapids](https://grandrapids.slack.com/) workspaces.

#### Seagate's Paragon NTFS driver

<!-- TODO Reword this -->

Visit https://www.seagate.com/support/software/paragon/ and install *Paragon Driver for macOS (Big Sur and later)*.

This is difficult to automate because it's distributed as a DMG with an app bundle in there with a scripted install.

#### Google Earth Pro

On the old computer, select *My Places* > *Save Place As…* and choose the default file name `My Places.kmz`. AirDrop this file to the new computer and double-click it to import (*File* > *Import…* does not work). It will be imported under *Temporary Places* > *My Places.kmz* > *My Places*. Select all of the top-level places and drag then into *My Places* proper. Then delete the now-empty *My Places.kmz*. Done!
