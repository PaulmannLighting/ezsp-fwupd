use std::io::{ErrorKind, Result};

/// Trait to ignore [`ErrorKind::TimedOut`] errors in a [`Result`].
pub trait IgnoreTimeout<T> {
    /// Ignores `TimedOut` errors.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(value))` if the result is `Ok(value)`, and `Ok(None)` if the result is an error of kind `TimedOut`.
    ///
    /// # Errors
    ///
    /// Returns an [`std::io::Error`] if an error occurs other than `TimedOut`.
    fn ignore_timeout(self) -> Result<Option<T>>;
}

impl<T> IgnoreTimeout<T> for Result<T> {
    fn ignore_timeout(self) -> Result<Option<T>> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(error) if error.kind() == ErrorKind::TimedOut => Ok(None),
            Err(error) => Err(error),
        }
    }
}
