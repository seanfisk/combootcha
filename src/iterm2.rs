use anyhow::Result;
use log::info;
use serde_json::json;
use users::{os::unix::UserExt, User};

use std::path::Path;

use crate::path::PathExt;
use crate::user::UserExt as OtherUserExt;

pub(crate) fn configure(standard_user: &User) -> Result<()> {
    let app_support_dir = standard_user
        .home_dir()
        .join(Path::new("Library/Application Support/iTerm2"));

    let bgs_dir = app_support_dir.join("Backgrounds");

    let dynamic_profiles_dir = app_support_dir.join("DynamicProfiles");
    // This file contains profiles used as parents by the iTerm2/fasd
    // integration. Since iTerm2 loads the list of dynamic profiles
    // alphabetically, we prefix it with a hyphen to ensure it is loaded first.
    // https://iterm2.com/documentation-dynamic-profiles.html
    let personal_profiles_path = dynamic_profiles_dir.join("-Personal.json");

    info!(
        "Installing iTerm2 dynamic profiles to {:?}",
        personal_profiles_path.to_string_lossy()
    );
    let personal_profile_guid = "411F060B-E097-4E29-9986-275D5A47F609";
    let personal_profile_name = "Personal";

    // TODO Restore the volcano image on root shell

    let profiles_json = json!({
        "Profiles": [
            {
                "Guid": personal_profile_guid,
                // General
                "Name": personal_profile_name,
                // Text
                "Cursor Type": 2, // Box cursor
                "Blinking Cursor": false,
                "Normal Font": make_font(20),
                "Use Non-ASCII Font" : false, // Use the same font for non-ASCII text
                "Ambiguous Double Width": false,
                // Window
                "Background Image Location": bgs_dir.join("holland-beach-sunset.jpg").to_str_safe()?,
                "Blend": 0.4,
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
            },
            {
                "Guid": "4A0A1F6D-753F-4D35-B019-F63C3144CC99",
                "Dynamic Profile Parent Name": personal_profile_name,
                // General
                "Name": "Presenter Mode",
                // Text
                "Normal Font": make_font(36),
            },
            {
                "Guid": "4381BB8C-7F7D-4CFD-A5F8-3F1A77185E37",
                "Dynamic Profile Parent Name": personal_profile_name,
                // General
                "Name": "System",
                // Window
                "Background Image Location": "/Library/Desktop Pictures/Mojave Night.jpg"
            }
        ]
    });

    standard_user.as_user(|| {
        crate::fs::ensure_dir(&bgs_dir)?;
        crate::fs::ensure_dir(&dynamic_profiles_dir)?;

        let file = crate::fs::create_file(&personal_profiles_path)?;
        serde_json::to_writer_pretty(file, &profiles_json)?;
        Ok(())
    })
}

fn make_font(size: u32) -> String {
    format!("InconsolataForPowerline {}", size)
}
