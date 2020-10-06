use anyhow::Result;
use log::info;
use serde_json::json;
use users::{os::unix::UserExt, User};

use std::path::Path;

use crate::path::PathExt;

pub(crate) fn configure(standard_user: &User) -> Result<()> {
    let app_support_dir = standard_user
        .home_dir()
        .join(Path::new("Library/Application Support/iTerm2"));

    info!("Creating iTerm2 backgrounds directory");
    let bgs_dir = app_support_dir.join("Backgrounds");
    crate::fs::ensure_dir_with_owner(&bgs_dir, &standard_user)?;

    info!("Installing iTerm2 dynamic profiles");
    let dynamic_profiles_dir = app_support_dir.join("DynamicProfiles");
    let personal_profile_guid = "411F060B-E097-4E29-9986-275D5A47F609";
    let personal_profile_name = "Personal";

    // TODO Restore the volcano image on root shell
    // TODO Restore system profile?

    let profiles = vec![
        json!({
            "Guid": personal_profile_guid,
            // General
            "Name": personal_profile_name,
            // Text
            "Cursor Type": 2, // Box cursor
            "Blinking Cursor": false,
            "Normal Font": make_font(20),
            "Ambiguous Double Width": false,
            // Window
            "Background Image Location": bgs_dir.join("holland-beach-sunset.jpg").to_str_safe()?,
            "Blend": 0.4,
            "Sync Title": true,
            // Terminal
            "Character Encoding": 4, // UTF-8
            "Terminal Type": "xterm-256color",
            "Set Local Environment Vars": true, // This means *Locale*, not *Local*
            "Place Prompt at First Column": true,
            "Show Mark Indicators": true,
            // Session
            "Close Sessions On End": true,
            "Prompt Before Closing 2": 0, // Do not prompt before closing
            // Keys
            "Option Key Sends": 2, // Esc+
            "Right Option Key Sends": 2, // Esc+

        }),
        json!({
            "Guid": "4A0A1F6D-753F-4D35-B019-F63C3144CC99",
            "Dynamic Profile Parent Name": personal_profile_name,
            // General
            "Name": "Presenter Mode",
            // Text
            "Normal Font": make_font(36),
        }),
    ];

    Ok(())
}

fn make_font(size: u32) -> String {
    format!("InconsolataForPowerline {}", size)
}
