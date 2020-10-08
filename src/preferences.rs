use anyhow::Result;
use users::User;

use crate::user::UserExt;
use crate::user_defaults::App;

pub(crate) fn set(standard_user: &User) -> Result<()> {
    standard_user.as_user(|| {
        App::new("com.bluemedora.vROps Deploy")?
            .bool("TestKey", false)?
            .sync()?;

        Ok(())
    })
}
