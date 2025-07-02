use std::borrow::Cow;

use indicatif::ProgressBar;
use log::info;

/// Trait for displaying and managing progress during firmware updates.
pub trait FlashProgress {
    /// Sets the message to be displayed on the progress bar.
    fn set_message(&self, msg: impl Into<Cow<'static, str>>);

    /// Increases the progress bar by one step.
    fn increase(&self);

    /// Prints a message to the progress bar or logs it if no progress bar is available.
    fn println(&self, msg: impl AsRef<str>);
}

impl FlashProgress for Option<&ProgressBar> {
    fn set_message(&self, msg: impl Into<Cow<'static, str>>) {
        if let Some(progress_bar) = self {
            progress_bar.set_message(msg);
        } else {
            info!("{}", msg.into());
        }
    }

    fn increase(&self) {
        if let Some(progress_bar) = self {
            progress_bar.inc(1);
        }
    }

    fn println(&self, msg: impl AsRef<str>) {
        if let Some(progress_bar) = self {
            progress_bar.println(msg);
        } else {
            info!("{}", msg.as_ref());
        }
    }
}
