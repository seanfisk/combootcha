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

        // Show percentage on battery indicator.
        //
        // Note: For some reason, Apple chose the value of ShowPercent to be YES
        // or NO as a string instead of using a Boolean.
        App::new("com.apple.menuextra.battery")?
            .string("ShowPercent", "YES")?
            .sync()?;

        // Start the character viewer in docked mode. The large window mode doesn't
        // take focus automatically, and can't AFAIK be focused with any keyboard
        // shortcut, rendering it less useful for those who like to stay on the
        // keyboard. The docked mode puts the cursor right in the search field, which
        // is perfect for keyboard users like myself.
        App::new("com.apple.CharacterPaletteIM")?
            .bool("CVStartAsLargeWindow", false)?
            .sync()?;

        App::new("com.apple.iChat")? // Messages.app
            .bool("SaveConversationsOnClose", true)? // Save history when conversations are closed
            .sync()?;

        Ok(())
    })
}
