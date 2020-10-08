use anyhow::Result;
use users::User;

use crate::user::UserExt;
use crate::user_defaults::App;

pub(crate) fn set(standard_user: &User) -> Result<()> {
    standard_user.as_user(|| {
        // Set up clock with day of week, date, and 24-hour clock.
        App::new("com.apple.menuextra.clock")?
            .string("DateFormat", "EEE MMM d  H:mm")?
            .bool("FlashDateSeparators", false)?
            .bool("IsAnalog", false)?
            .sync()?;

        Ok(())
    })
}
