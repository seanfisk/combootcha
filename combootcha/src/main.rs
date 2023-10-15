use anyhow::Result;
use log::info;

fn main() -> Result<()> {
    let standard_user = scoby::run_shared_setup(
        /*brewfile_extra_bytes=*/ include_bytes!("Brewfile"),
        /*ssh_config_extra_bytes=*/ include_bytes!("ssh-config"),
        /*git_email=*/ "sean@seanfisk.com"
    )?;

    standard_user.as_effective_user(|| {
        let lastpass_cmd_shift_key = "1179914";
        scoby::user_defaults::App::new("com.lastpass.LastPass")?
        // Some preferences are prefixed by a hash, which seems to be stored in
        // 'lp_local_pwhash'. We don't know what that hash means, or whether it's
        // consistent, so just leave those alone.
            .string("global_StartOnLogin", "1")?
        // ⌘⇧L
            .string("global_SearchHotKeyMod", lastpass_cmd_shift_key)?
            .string("global_SearchHotKeyVK", "37")?
        // ⌘⇧V
            .string("global_VaultHotKeyMod", lastpass_cmd_shift_key)?
            .string("global_VaultHotKeyVK", "9")?
            .sync()?;
    })?;

    info!("Setup complete!");
}
