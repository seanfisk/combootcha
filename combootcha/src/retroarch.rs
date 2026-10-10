use anyhow::Result;
use users::User;

use scoby::UserExt;

pub(crate) fn configure(standard_user: &User) -> Result<()> {
    // RetroArch uses a bespoke configuration format. Wonderful. Find the details here:
    // https://github.com/libretro/libretro-common/blob/master/file/config_file.c
    // https://github.com/libretro/libretro-common/blob/master/include/file/config_file.h
    //
    // When RetroArch starts with the config file
    // Missing: ???
    // Empty: ???
    // Some keys missing: ??


    standard_user.as_effective_user(|| Ok(()))
}
