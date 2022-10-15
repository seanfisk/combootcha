use anyhow::Result;
use users::{os::unix::UserExt, User};

use serde_json::json;

use crate::user::UserExt as OtherUserExt;

pub(crate) fn configure(standard_user: &User) -> Result<()> {
    let config_dir = standard_user.home_dir().join(".config/karabiner");
    let json = json!({
        "profiles": [
            {
                "name": "Default",
                "selected": true,
                // Swap Caps Lock and Control
                "simple_modifications": [
                    {
                        "from": { "key_code": "left_control" },
                        "to": { "key_code": "caps_lock" }
                    },
                    {
                        "from": { "key_code": "caps_lock" },
                        "to": { "key_code": "left_control" }
                    },
                ],
                "complex_modifications": {
                    "rules": [
                        { "description": "Pressing spacebar inserts space. Holding spacebar holds control. Disabled in RetroArch.",
                           "manipulators": [
                               { "from": { "key_code": "spacebar", "modifiers": { "optional": ["any"] } },
                                  "to": [{ "key_code": "left_control" }],
                                  "to_if_alone": [{ "key_code": "spacebar" }],
                                  "type": "basic",
                                  // I use spacebar for the speed toggle in RetroArch and the space to ctrl mapping does not work well with this
                                  // See https://github.com/pqrs-org/Karabiner-Elements/issues/1109
                                  "conditions": [
                                      {
                                          "type": "frontmost_application_unless",
                                          "bundle_identifiers": ["^libretro\\.RetroArch$"]
                                      }
                                  ]
                               },
                           ],
                        },
                    ],
                },
                "devices": [
                    {
                        "identifiers": {
                            "is_keyboard": true,
                            "is_pointing_device": false,
                            // Filco Majestouch 2
                            "product_id": 17_733,
                            "vendor_id": 1_241,
                        },
                        // Swap Option and Command
                        "simple_modifications": [
                            {
                                "from": { "key_code": "left_option" },
                                "to": { "key_code": "left_command" }
                            },
                            {
                                "from": { "key_code": "left_command" },
                                "to": { "key_code": "left_option" }
                            },
                            {
                                "from": { "key_code": "right_option" },
                                "to": { "key_code": "right_command" }
                            },
                            {
                                "from": { "key_code": "right_command" },
                                "to": { "key_code": "right_option" }
                            },
                        ],
                    },
                ],
            }
        ]
    });

    let path = config_dir.join("karabiner.json");
    standard_user.as_effective_user(|| {
        crate::fs::ensure_dir(&config_dir)?;
        let file = crate::fs::create_file(&path)?;
        serde_json::to_writer_pretty(file, &json)?;
        Ok(())
    })
}
