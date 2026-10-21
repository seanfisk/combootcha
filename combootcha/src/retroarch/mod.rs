mod config_file;

use anyhow::{Context, Result};
use log::{debug, info};
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use tempfile::NamedTempFile;
use users::{os::unix::UserExt, User};

use config_file::Config;

use scoby::UserExt as OtherUserExt;

const ENTRIES: &[(&str, &str)] = &[
    // General options

    // Note: video_fullscreen stores the *last state* of whether the app was fullscreen, so it's not particularly useful to set here

    // Disable the default menu navigation bindings: https://docs.libretro.com/guides/input-and-controls/#general-controller-mapping
    //
    // Unified Menu Controls defaulting to OFF solves a chicken and egg problem: it enables a user to navigate the menu before having a RetroPad configured. Once a RetroPad is configured, the RetroPad binds can be used to navigate the menu.
    ("menu_unified_controls", "true"),
    ("input_autodetect_enable", "false"), // Allows clearing RetroPad bindings for analog sticks without falling back to automatic bindings
    ("menu_swap_ok_cancel_buttons", "false"), // RetroPad A = OK, RetroPad B = Cancel
    ("confirm_quit", "true"),
    // If I had hotkeys for these actions, I'd enable confirmation. But when confirmation is enabled, it still requires a second key press on even the actions from the Quick Menu. This is just annoying.
    ("confirm_close", "false"),
    ("confirm_reset", "false"),
    // RetroPad bindings
    // Order matches RetroArch's UI
    // A value is explicitly set to "nul" to remove an unwanted RetroArch default binding
    //   Keyboard
    ("input_player1_b", "k"),
    ("input_player1_a", "j"),
    ("input_player1_y", "nul"),
    ("input_player1_x", "nul"),
    ("input_player1_select", "y"),
    ("input_player1_start", "l"),
    ("input_player1_l", "u"),
    ("input_player1_r", "i"),
    ("input_player1_up", "s"),
    ("input_player1_down", "d"),
    ("input_player1_left", "a"),
    ("input_player1_right", "f"),
    //     Hotkeys
    ("input_reset", "nul"),
    ("input_toggle_fast_forward", "nul"),
    ("input_hold_fast_forward", "space"),
    ("input_toggle_slowmotion", "nul"),
    ("input_hold_slowmotion", "nul"),
    ("input_rewind", "nul"),
    ("input_pause_toggle", "nul"),
    ("input_frame_advance", "nul"),
    ("input_audio_mute", "nul"),
    ("input_volume_up", "equals"),
    ("input_volume_down", "minus"),
    ("input_menu_toggle", "q"),
    ("input_load_state", "m"),
    ("input_save_state", "comma"),
    ("input_state_slot_increase", "slash"),
    ("input_state_slot_decrease", "period"),
    ("input_shader_toggle", "nul"),
    ("input_shader_next", "nul"),
    ("input_shader_prev", "nul"),
    ("input_cheat_toggle", "nul"),
    ("input_cheat_index_plus", "nul"),
    ("input_cheat_index_minus", "nul"),
    ("input_screenshot", "nul"),
    ("input_grab_mouse_toggle", "nul"),
    ("input_game_focus_toggle", "nul"),
    ("input_toggle_fullscreen", "w"),
    ("input_desktop_menu_toggle", "nul"),
    ("input_fps_toggle", "nul"),
    ("input_netplay_game_watch", "nul"),
    ("input_netplay_player_chat", "nul"),
    ("input_menu_search", "r"),
    //   XBOX One Wireless Controller bindings
    //   Button names come from here: https://support.xbox.com/en-US/help/hardware-network/controller/xbox-one-wireless-controller
    //
    //     RetroPad binds
    ("input_player1_analog_dpad_mode", "0"), // Set Analog to Digital Type to None. This is important because we want to use the D-pad for hotkeys.
    // The button positions on the RetroPad do not match the button positions on the XBOX One Wireless Controller. This layout is deliberate as I want to be able to have the tip of my thumb on the RetroPad B button and be able to rock my thumb to activate the RetroPad A button. This causes the displayed button positions in the bottom right of the RetroArch menu to be incorrect. I've decided to live with that as there is no readily apparent way to fix it without altering RetroArch's source code, and I'd rather have this incorrect than settle for a non-optimal button layout while playing the games.
    ("input_player1_b_btn" /*bottom*/, "1"), // XBOX One X button
    ("input_player1_a_btn" /*right*/, "0"),  // XBOX One A button
    ("input_player1_y_btn" /*left*/, "9"),   // XBOX One Y button
    ("input_player1_x_btn" /*top*/, "8"),    // XBOX One B button
    ("input_player1_select_btn", "2"),       // XBOX One View button
    ("input_player1_start_btn", "3"),        // XBOX One Menu button
    ("input_player1_l_btn" /*shoulder*/, "12"), // XBOX One left trigger
    ("input_player1_r_btn" /*shoulder*/, "13"), // XBOX One right trigger
    //       Unbind because the games I am playing don't use them, and I am using them for hotkeys
    ("input_player1_l2_btn" /*trigger*/, "nul"),
    ("input_player1_r2_btn" /*trigger*/, "nul"),
    ("input_player1_l3_btn" /*thumb*/, "nul"),
    ("input_player1_r3_btn" /*thumb*/, "nul"),
    ("input_player1_up_axis", "+1"),    // Left stick up
    ("input_player1_down_axis", "-1"),  // Left stick down
    ("input_player1_left_axis", "-0"),  // Left stick left
    ("input_player1_right_axis", "+0"), // Left stick right
    //       Unbind because I'm not using a gun
    ("input_player1_gun_trigger_mbtn", "nul"),
    (
        "input_player1_gun_offscreen_shot_mbtn", /*reload*/
        "nul",
    ),
    ("input_player1_gun_start", "nul"),
    ("input_player1_gun_start_mbtn", "nul"),
    ("input_player1_gun_select", "nul"),
    ("input_player1_gun_select_mbtn", "nul"),
    //     Hotkeys
    ("input_hold_fast_forward_btn", "11"),  // Right bumper
    ("input_hold_slowmotion_btn", "10"),    // Left bumper
    ("input_audio_mute_axis", "-2"),        // Right stick left
    ("input_volume_up_axis", "+3"),         // Right stick up
    ("input_volume_down_axis", "-3"),       // Right stick down
    ("input_menu_toggle_btn", "16"),        // XBOX One XBOX button
    ("input_load_state_btn", "4"),          // D-pad up
    ("input_save_state_btn", "5"),          // D-pad down
    ("input_state_slot_increase_btn", "7"), // D-pad right
    ("input_state_slot_decrease_btn", "6"), // D-pad left
    ("input_toggle_fullscreen_btn", "15"),  // Right stick click
];

pub(crate) fn configure(standard_user: &User) -> Result<()> {
    // Upon start, RetroArch reads retroarch.cfg into memory. If retroarch.cfg is missing, empty, or missing some keys, the missing keys will get default values and immediately be written back.
    //
    // Upon quit, RetroArch always writes retroarch.cfg from memory with no regard to any changes that were made to the file externally while RetroArch was running.
    //
    // When settings are changed within RetroArch, the application stores them in memory but does not write them back to retroarch.cfg until quit. This means that the application must be quit cleanly for changes to be written.
    standard_user.as_effective_user(|| {
        let config_dir = standard_user
            .home_dir()
            .join("Library/Application Support/RetroArch/config");
        scoby::fs::ensure_dir(&config_dir)?;
        let config_path = config_dir.join("retroarch.cfg");
        info!("Reading RetroArch config from {:?}", config_path);
        let (mut config, original_permissions) = || -> Result<_> {
            match File::open(&config_path) {
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        debug!("Config file {config_path:?} does not exist; starting with empty config");
                        Ok((Config::default(), std::fs::Permissions::from_mode(0o644)))
                    } else {
                        Err(e.into())
                    }
                }
                Ok(file) => {
                    let config = Config::read_from(&file)?;
                    let permissions = file.metadata()?.permissions();
                    Ok((config, permissions))
                }
            }
        }().with_context(|| {
            format!("Could not read RetroArch config from {config_path:?}")
        })?;
        let mut updated = false;
        for &(key, value) in ENTRIES {
            updated |= config.replace(key, value)?;
        }
        if updated {
            info!("Writing updated RetroArch config to {config_path:?}");

            || -> Result<()> {
                // Create temp file in same directory as original file. This guarantees the files will be on the same filesystem, so the rename will be atomic.
                let temp_file = NamedTempFile::new_in(&config_dir)
                    .with_context(|| format!("Could not create temporary file in {:?}", config_dir))?;
                let temp_file_path = temp_file.path();
                debug!("Writing new RetroArch config to temp file {temp_file_path:?}");
                config.write_to(&temp_file)?;
                temp_file.as_file()
                    .sync_all()
                    .with_context(|| format!("Could not sync temporary file at {:?}", temp_file_path))?;

                debug!("Applying original permissions to {temp_file_path:?}");
                temp_file.as_file().set_permissions(original_permissions)
                    .with_context(|| format!("Error applying original permissions to {config_path:?}"))?;

                debug!("Moving temp file {temp_file_path:?} to {config_path:?}");
                let temp_file_path = temp_file_path.to_owned(); // Rust variable `temp_file` is moved when we call `persist` (I am not talking about being moved on the filesystem)
                temp_file.persist(&config_path).with_context(|| format!("Could not move temporary file {temp_file_path:?} to {config_path:?}"))?;

                Ok(())
            }().with_context(|| format!("Could not write RetroArch config to {config_path:?}"))?;
        } else {
            info!("RetroArch already configured correctly; not updating config file");
        }
        Ok(())
    })
}
