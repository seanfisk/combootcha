use anyhow::Result;
use users::User;

use scoby::UserExt;

pub(crate) fn configure(standard_user: &User) -> Result<()> {
    let delete_key = "";
    let key_code = 51;
    let sleep_hotkey_dict = make_hotkey_dict(delete_key, key_code, 1_572_864);
    let eject_hotkey_dict = make_hotkey_dict(delete_key, key_code, 1_703_936);

    standard_user.as_effective_user(|| {
        scoby::user_defaults::App::new("com.stclairsoft.Jettison")?
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
            // Frequently I will run Time Machine, and then my computer will sleep. I'd rather not have the disk remounted when it wakes up. I can always manually remount it.
            .bool("remountOnWake", false)?
            .bool("showRemountProgress", false)?
            // Set "Eject disks and sleep" hotkey to ⌘⌥⌫
            .dict("sleepHotkey", &sleep_hotkey_dict)?
            .bool("statusItemEnabled", true)?
            .bool("toggleMassStorageDriver", false)?
            // Ejection is preferable as it will turn off the light in my current Time Machine drive and the disk will stop spinning.
            .bool("unmountNotEject", false)?
            .sync()
    })
}

fn make_hotkey_dict(
    characters: &str,
    key_code: i64,
    modifier_flags: i64,
) -> std::collections::HashMap<&str, scoby::user_defaults::DictValue> {
    use scoby::user_defaults::DictValue;
    std::collections::HashMap::from([
        ("characters", DictValue::String(characters)),
        ("charactersIgnoringModifiers", DictValue::String(characters)),
        ("keyCode", DictValue::Int(key_code)),
        ("modifierFlags", DictValue::Int(modifier_flags)),
    ])
}
