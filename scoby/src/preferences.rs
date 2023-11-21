use anyhow::{Context, Result};
use std::io::Cursor;
use users::User;

use crate::user_defaults::App;
use crate::verbose_command::Command;
use crate::UserExt;

pub(crate) fn set(standard_user: User) -> Result<()> {
    // Any preferences that don't already have specific install instructions go here.

    // Apps such as iTerm2 which have specific instructions in this program should set their preferences in their specific function/file.

    standard_user.as_effective_user(|| {
        // Set up clock with day of week, date, and 24-hour clock.
        App::new("com.apple.menuextra.clock")?
            .string("DateFormat", "EEE MMM d  H:mm")?
            .bool("FlashDateSeparators", false)?
            .bool("IsAnalog", false)?
            .sync()?;

        // Start the character viewer in docked mode. The large window mode doesn't
        // take focus automatically, and can't AFAIK be focused with any keyboard
        // shortcut, rendering it less useful for those who like to stay on the
        // keyboard. The docked mode puts the cursor right in the search field, which
        // is perfect for keyboard users like myself.
        App::new("com.apple.CharacterPaletteIM")?
            .bool("CVStartAsLargeWindow", false)?
            .sync()?;

        // com.lightheadsw.caffeine was the old key for the original Caffeine. If you find this, delete it: it is not used anymore.
        App::new("com.intelliscapesolutions.caffeine")?
            .bool("ActivateOnLaunch", false)? // Don't activate on app launch, since we now run Caffeine at login
            .int("DefaultDuration", 60)? // Activate for an hour by default
            .bool("SuppressLaunchMessage", true)? // Don't show the welcome banner
            .sync()?;

        // Console and Monitor themes themselves seem not to be stored in preferences.
        App::new("com.secretgeometry.Cathode")?
            .bool("CloseOnExit", true)?
            .bool("JitterWhenWindowMoves", true)?
            .bool("PositionalPerspective", true)?
            .int("RenderingQuality", 3)? // High
            .bool("UseColorPalette", true)?
            .bool("UseOptionAsMeta", true)?
            .bool("UseSounds", false)?
            .sync()?;

        App::new("com.titanium.Deeper")?
            .bool("ConfirmQuit", false)?
            .bool("ConfirmQuitApp", true)?
            .bool("DeleteLog", true)?
            .bool("DrawerEffect", true)?
            .bool("Licence", false)? // Don't show the license at startup
            .bool("OpenLog", false)?
            .bool("ShowHelp", false)?
            .sync()?;

        {
            let delete_key = "";
            let key_code = 51;
            let sleep_hotkey_dict = make_jettison_hotkey_dict(delete_key, key_code, 1_572_864);
            let eject_hotkey_dict = make_jettison_hotkey_dict(delete_key, key_code, 1_703_936);

            App::new("com.stclairsoft.Jettison")?
                .bool("askedToLaunchAtLogin", true)? // We use launchd to start Jettison at login
                .bool("autoEjectAtLogout", false)?
                .bool("autoEjectEnabled", true)? // This really means autoEjectAtSleep
                .bool("ejectDiskImages", true)?
                .bool("ejectHardDisks", true)?
                // Set "Eject external disks" hotkey to ⌘⇧⌥⌫
                .dict("ejectHotkey", &eject_hotkey_dict)?
                .bool("ejectNetworkDisks", true)?
                .bool("ejectOpticalDisks", false)?
                .bool("askedAboutSDCards", true)? // We are telling the app right below
                .bool("ejectSDCards", false)?
                .bool("hideMenuBarIcon", false)?
                .bool("moveToApplicationsFolderAlertSuppress", true)?
                .bool("playSoundOnFailure", false)?
                .bool("playSoundOnSuccess", false)?
                .bool("showRemountProgress", false)?
                // Set "Eject disks and sleep" hotkey to ⌘⌥⌫
                .dict("sleepHotkey", &sleep_hotkey_dict)?
                .bool("statusItemEnabled", true)?
                .bool("toggleMassStorageDriver", false)?
                .sync()?;
        }

        App::new("com.apple.screensaver")?
            .bool("askForPassword", false)?
            .int("askForPasswordDelay", 5)?
            .sync()?;

        App::new("com.apple.screencapture")?
            .string("type", "PNG")?
            .sync()?;

        App::new("org.macosforge.xquartz.X11")?
            // Input
            .bool("enable_fake_buttons", false)?
            .bool("sync_keymap", false)?
            .bool("enable_key_equivalents", true)?
            .bool("option_sends_alt", true)?
            // Output
            .bool("rootless", true)?
            .bool("fullscreen_menu", true)?
            // Pasteboard
            //   Syncing is somewhat broken, see here:
            //   <http://xquartz.macosforge.org/trac/ticket/765>
            //   If you go into XQuartz and press ⌘C it will usually sync it.
            .bool("sync_pasteboard", true)?
            .bool("sync_clipboard_to_pasteboard", true)?
            .bool("sync_pasteboard_to_clipboard", true)?
            .bool("sync_pasteboard_to_primary", true)?
            .bool("sync_primary_on_select", false)?
            // Windows
            .bool("wm_click_through", false)?
            .bool("wm_ffm", false)?
            .bool("wm_focus_on_new_window", true)?
            // Security
            .bool("no_auth", false)?
            .bool("nolisten_tcp", true)?
            // Other
            // XXX seems to do nothing, xterm still starts /bin/sh
            //.string("login_shell", "/path/to/zsh")?
            .sync()?;

        // Tweaks from
        // https://github.com/kevinSuttle/OSXDefaults/blob/master/.osx
        // https://github.com/mathiasbynens/dotfiles/blob/master/.osx

        // A note on settings: if the value is zero, set it as an integer 0 instead of
        // float 0.0. Otherwise, it will be "cast" to a float by the defaults system
        // and the resource will be updated every time.
        App::new("NSGlobalDomain")?
            // Always show scrollbars
            .string("AppleShowScrollBars", "Always")?
            // Allow keyboard access to all controls (using Tab), not just text boxes and lists.
            .int("AppleKeyboardUIMode", 2)?
            // Increase window resize speed for Cocoa applications
            .float("NSWindowResizeTime", 0.001)?
            // Expand save panel by default
            .bool("NSNavPanelExpandedStateForSaveMode", true)?
            .bool("NSNavPanelExpandedStateForSaveMode2", true)?
            // Expand print panel by default
            .bool("PMPrintingExpandedStateForPrint", true)?
            .bool("PMPrintingExpandedStateForPrint2", true)?
            // Save to disk (not to iCloud) by default
            .bool("NSDocumentSaveNewDocumentsToCloud", false)?
            // Disable natural scrolling
            .bool("com.apple.swipescrolldirection", false)?
            // Display ASCII control characters using caret notation in standard text
            // views
            // Try e.g. `cd /tmp; echo -e '\x00' > cc.txt; open -e cc.txt`
            .bool("NSTextShowsControlCharacters", true)?
            // Disable press-and-hold for keys in favor of key repeat
            .bool("ApplePressAndHoldEnabled", false)?
            // Key repeat
            //   Set a keyboard repeat rate to fast
            .int("KeyRepeat", 2)?
            //   Set low initial delay
            .int("InitialKeyRepeat", 15)?
            // Trackpad (other trackpad settings are in com.apple.AppleMultitouchTrackpad)
            // TODO Having trouble getting these settings to take effect, even after a restart
            //   Speed
            .string("com.apple.trackpad.scaling", "0.6875")?
            //   Click
            .bool("com.apple.trackpad.forceClick", true)?
            // Finder
            //   Show all filename extensions
            .bool("AppleShowAllExtensions", true)?
            //   Enable spring loading for directories
            // .bool("com.apple.springing.enabled", true)?
            // Remove the spring loading delay for directories
            // .int("com.apple.springing.delay", 0)?
            .sync()?;

        // TODO Having trouble getting these settings to take effect, even after a restart
        App::new("com.apple.AppleMultitouchTrackpad")?
            // Disable "Tap to click"
            .bool("Clicking", false)?
            // TODO Consider audible clicking
            // Silent clicking
            .int("ActuationStrength", 0)?
            .sync()?;

        // Automatically quit printer app once the print jobs complete
        App::new("com.apple.print.PrintingPrefs")?
            .bool("Quit When Finished", true)?
            .sync()?;

        // Set Help Viewer windows to non-floating mode
        // App::new("com.apple.helpviewer")?
        //     .bool("DevMode", true)?
        //     .sync()?;

        // TODO
        // Reveal IP address, hostname, OS version, etc. when clicking the clock in the
        // login window
        // App::new("/Library/Preferences/com.apple.loginwindow")?
        //         "AdminHostInfo" => "HostName",
        // .sync()?;

        // More Finder tweaks
        // Note: Quitting Finder will also hide desktop icons.
        App::new("com.apple.finder")?
            // Allow quitting via Command-Q
            .bool("QuitMenuItem", true)?
            // Disable window animations and Get Info animations
            .bool("DisableAllAnimations", true)?
            // Don't show hidden files by default -- this shows hidden files on the
            // desktop, which is just kind of annoying. I've haven't really seen other
            // benefits, since I don't use Finder much.
            .bool("AppleShowAllFiles", false)?
            // Show status bar
            .bool("ShowStatusBar", true)?
            // Show path bar
            .bool("ShowPathbar", true)?
            // Allow text selection in Quick Look
            .bool("QLEnableTextSelection", true)?
            // Display full POSIX path as Finder window title
            .bool("_FXShowPosixPathInTitle", true)?
            // When performing a search, search the current folder by default
            .string("FXDefaultSearchScope", "SCcf")?
            // Disable the warning when changing a file extension
            .bool("FXEnableExtensionChangeWarning", false)?
            // Use list view in all Finder windows by default
            // Four-letter codes for the other view modes: `icnv`, `clmv`, `Flwv`
            .string("FXPreferredViewStyle", "Nlsv")?
            .sync()?;

        // Avoid creating .DS_Store files on network
        App::new("com.apple.desktopservices")?
            .bool("DSDontWriteNetworkStores", true)?
            .sync()?;

        // App::new("com.apple.NetworkBrowser")?
        //     // Enable AirDrop over Ethernet
        //     .bool("BrowseAllInterfaces", true)?
        //     .sync()?;

        App::new("com.apple.dock")?
            // Remove the auto-hiding Dock delay
            .int("autohide-delay", 0)?
            // Remove the animation when hiding/showing the Dock
            .int("autohide-time-modifier", 0)?
            // Automatically hide and show the Dock
            .bool("autohide", true)?
            // Make Dock icons of hidden applications translucent
            .bool("showhidden", true)?
            .sync()?;

        // App::new("com.apple.TimeMachine")?
        //     // Prevent Time Machine from prompting to use new hard drives as backup volume
        //     .bool("DoNotOfferNewDisksForBackup", true)?
        //     .sync()?;

        // App::new("com.apple.TextEdit")?
        //     // Use plain text mode for new TextEdit documents
        //     .int("RichText", 0)?
        //     // Open and save files as UTF-8 in TextEdit
        //     .int("PlainTextEncoding", 4)?
        //     .int("PlainTextEncodingForWrite", 4)?
        //     .sync()?;

        App::new("com.apple.DiskUtility")?
            // Enable the debug menu in Disk Utility
            .bool("DUDebugMenuEnabled", true)?
            // Enable the advanced image menu in Disk Utility
            .bool("advanced-image-options", true)?
            .sync()?;

        // TODO Numerous settings within this block are causing defaults synchronization to fail for some reason. It seems to be somewhat non-deterministic but it is recurring.
        /*
        App::new("com.apple.universalaccess")?
            // All closeView keys control the screen zoom.
            //   'Zoom style' choices:
            //
            //       0. Fullscreen
            //       1. Picture-in-picture
            //
            //   Don't set this. Fullscreen is the default anyway, and this way we can
            //   let the user change based upon needs at that point. We have fullscreen
            //   and PIP settings later as well.
            // .int("closeViewZoomMode", 0)?
            .bool("closeViewHotkeysEnabled", false)?
            //   Use scroll gesture with modifier keys to zoom.
            // .bool("closeViewScrollWheelToggle", true)?
            //   Use Ctrl as the modifier key (the number is a key code or something).
            //   This seems not to work correctly (?).
            // .int("closeViewScrollWheelModifiersInt", 262_144)?
            .bool("closeViewSmoothImages", true)?
            //   Don't follow *keyboard* focus.
            .bool("closeViewZoomFollowsFocus", false)?
            //   Fullscreen zoom settings
            //     Choices: When zoomed in, the screen image moves:
            //
            //         0. Continuously with pointer
            //         1. Only when the pointer reaches an edge
            //         2. So the pointer is at or near the center of the screen
            .int("closeViewPanningMode", 1)?
            //   Picture-in-picture settings
            //     Use system cursor in zoom.
            .int("closeViewCursorType", 0)?
            //     Enable temporary zoom (with Ctrl-Cmd)
            .bool("closeViewPressOnReleaseOff", true)?
            //     Choices:
            //
            //         1. Stationary
            //         2. Follow mouse cursor
            //         3. Tiled along edge
            // .int("closeViewWindowMode", 1)?
            .sync()?;
        */

        {
            // These preferences are stored in an embedded binary plist data value at key dnd_prefs.
            // Source: https://github.com/tiiiecherle/osx_install_config/blob/933f82629dcf35b64d2e691983f3555f27ef560b/11_system_and_app_preferences/11c_macos_preferences_14.sh#L275-L297

            // Build the plist structure in memory
            let mut dict = plist::Dictionary::new();
            for (name, value) in [
                ("dndDisplayLock", true),
                ("dndDisplaySleep", true),
                ("dndMirrored", true),
                ("facetimeCanBreakDND", false),
                ("repeatedFacetimeCallsBreaksDND", false),
            ] {
                dict.insert(name.to_owned(), plist::Value::Boolean(value));
            }

            // Write it to an in-memory buffer
            let mut buffer = Cursor::new(Vec::new());
            plist::to_writer_binary(&mut buffer, &dict)
                .context("Could not write notification center plist to in-memory buffer")?;

            // Set the user default using Core Foundation native API
            App::new("com.apple.ncprefs")?
                .data("dnd_prefs", buffer.get_ref())?
                .sync()?;
        }

        Ok(())
    })?;

    Command::new("/usr/bin/killall")
        .arg("-v")
        // Needed for the Do Not Disturb changes to take effect. The process will automatically be restarted.
        .arg("usernoted")
        // Needed for the screen capture changes to take effect. The process will automatically be restarted.
        .arg("SystemUIServer")
        .user(standard_user)
        .run()?;

    Ok(())
}

fn make_jettison_hotkey_dict(
    characters: &str,
    key_code: i64,
    modifier_flags: i64,
) -> std::collections::HashMap<&str, crate::user_defaults::DictValue> {
    use crate::user_defaults::DictValue;
    std::collections::HashMap::from([
        ("characters", DictValue::String(characters)),
        ("charactersIgnoringModifiers", DictValue::String(characters)),
        ("keyCode", DictValue::Int(key_code)),
        ("modifierFlags", DictValue::Int(modifier_flags)),
    ])
}
