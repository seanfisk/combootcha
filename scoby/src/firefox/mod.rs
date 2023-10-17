use crate::UserExt;
use anyhow::Result;
use log::info;
use std::fs;
use std::path::Path;
use users::User;

// See https://support.mozilla.org/en-US/kb/customizing-firefox-using-autoconfig#w_setting-up-autoconfig
// Also see https://mozilla.github.io/policy-templates/ although it probably doesn't apply to what we want to do

pub(crate) fn configure(standard_user: &User) -> Result<()> {
    standard_user.as_effective_user(|| {
        // Do not attempt to ensure that any of these directories exist. They should exist by virtue of Firefox being installed, and if they don't exist, we don't want to create them.
        let resources_dir = Path::new("/Applications/Firefox.app/Contents/Resources");

        let autoconfig_path = resources_dir.join("defaults/pref/autoconfig.js");
        info!("Writing Firefox AutoConfig file to {autoconfig_path:?}");
        fs::write(autoconfig_path, include_bytes!("autoconfig.js"))?;

        let cfg_path = resources_dir.join("firefox.cfg");
        info!("Writing Firefox preferences file to {cfg_path:?}");
        fs::write(cfg_path, include_bytes!("firefox.cfg"))?;

        Ok(())
    })
}
