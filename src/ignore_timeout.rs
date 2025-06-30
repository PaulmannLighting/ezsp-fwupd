use std::io::Result;

pub trait IgnoreTimeout<T> {
    /// Ignores `TimedOut` errors.
    fn ignore_timeout(self) -> Result<Option<T>>;
}

impl<T> IgnoreTimeout<T> for Result<T> {
    fn ignore_timeout(self) -> Result<Option<T>> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(error) if error.kind() == std::io::ErrorKind::TimedOut => Ok(None),
            Err(error) => Err(error),
        }
    }
}
