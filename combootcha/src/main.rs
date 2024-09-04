use anyhow::Result;
use clap::{crate_authors, crate_description, crate_name};
use log::info;
use scoby::UserExt;

fn main() -> Result<()> {
    scoby::check_root()?;

    let app = clap::App::new(crate_name!())
        .about(crate_description!())
        .author(crate_authors!());
    let mut global_config = scoby::GlobalConfig::new()?;
    let app = global_config.configure_cli(app);
    let matches = app.get_matches();
    let (_standard_username, standard_user) = scoby::parse_standard_user(&matches)?;

    global_config.zsh.add_profile_content("# extra stuff\n");
    global_config
        .homebrew
        .add_global_brewfile_content(include_str!("Brewfile"));

    global_config.converge(
        &matches,
        standard_user.clone(),
        /*git_email=*/ "sean@seanfisk.com",
        /*hammerspoon_init_lua_extra_bytes=*/ None,
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

        Ok(())
    })?;

    info!("Setup complete!");

    Ok(())
}
