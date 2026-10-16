mod config_file;

use anyhow::{Context, Result};
use log::{debug, info};
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use tempfile::NamedTempFile;
use users::{os::unix::UserExt, User};

use config_file::Config;

use scoby::UserExt as OtherUserExt;

// TODO Final pass to improve logging and errors

const ENTRIES: &[(&str, &str)] = &[
    ("input_audio_mute", "nul"),
    ("input_cheat_index_minus", "nul"),
    ("input_cheat_index_plus", "nul"),
    ("input_cheat_toggle", "nul"),
    ("input_fps_toggle", "nul"),
    ("input_frame_advance", "nul"),
    ("input_game_focus_toggle", "nul"),
    ("input_grab_mouse_toggle", "nul"),
    ("input_hold_fast_forward", "nul"),
    ("input_hold_slowmotion", "nul"),
    ("input_load_state", "n"),
    ("input_menu_toggle", "q"),
    ("input_netplay_game_watch", "nul"),
    ("input_netplay_player_chat", "nul"),
    ("input_pause_toggle", "nul"),
    ("input_player1_a", "k"),
    ("input_player1_b", "j"),
    ("input_player1_down", "d"),
    ("input_player1_gun_offscreen_shot_mbtn", "nul"),
    ("input_player1_gun_select", "nul"),
    ("input_player1_gun_select_mbtn", "nul"),
    ("input_player1_gun_start", "nul"),
    ("input_player1_gun_start_mbtn", "nul"),
    ("input_player1_gun_trigger_mbtn", "nul"),
    ("input_player1_l", "u"),
    ("input_player1_left", "a"),
    ("input_player1_r", "i"),
    ("input_player1_right", "f"),
    ("input_player1_select", "y"),
    ("input_player1_start", "h"),
    ("input_player1_up", "s"),
    ("input_reset", "nul"),
    ("input_rewind", "nul"),
    ("input_save_state", "h"),
    ("input_screenshot", "nul"),
    ("input_shader_next", "nul"),
    ("input_shader_prev", "nul"),
    ("input_shader_toggle", "nul"),
    ("input_state_slot_decrease", "period"),
    ("input_state_slot_increase", "comma"),
    ("input_toggle_fast_forward", "nul"),
    ("input_toggle_fullscreen", "nul"),
    ("input_volume_down", "minus"),
    ("input_volume_up", "equals"),
    ("menu_unified_controls", "true"),
    ("video_fullscreen", "true"),
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

            // Create temp file in same directory as original file. This guarantees the files will be on the same filesystem, so the rename will be atomic.
            let file = NamedTempFile::new_in(&config_dir)
                .with_context(|| format!("Could not create temporary file in {:?}", config_dir))?;
            let file_path = file.path();
            config.write_to(&file)?;
            file.as_file()
                .sync_all()
                .with_context(|| format!("Could not sync temporary file at {:?}", file_path))?;

            debug!("Moving output file into original location");
            let file_path = file_path.to_owned(); // Rust variable `file' is moved when we call `persist' (I am not talking about being moved on the filesystem)
            let file = file.persist(&config_path).with_context(|| {
                format!("Could not move temporary file {file_path:?} back to original file's location at {config_path:?}")
            })?;

            debug!("Setting output file permissions");
            file.set_permissions(original_permissions)
                .with_context(|| {
                    format!("Error writing permissions to newly-written output file {config_path:?}")
                })?;
        } else {
            info!("RetroArch already configured correctly; not updating config file");
        }
        Ok(())
    })
}
