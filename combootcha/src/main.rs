use anyhow::Result;
use log::info;
use scoby::UserExt;

fn main() -> Result<()> {

    let (scoby_cli, app) = scoby::Cli::init()?;
    let matches = app.get_matches();
    let mut global_config = scoby_cli.parse_config(&matches)?;
    let standard_user = global_config.standard_user.clone();

    global_config.zsh.add_profile_content("# extra stuff\n");
    global_config
        .homebrew
        .add_global_brewfile_content(include_str!("Brewfile"));
    global_config.git.set_email("sean@seanfisk.com");

    global_config.converge(&matches)?;

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

        Ok(())
    })?;

    info!("Setup complete!");

    Ok(())
}
