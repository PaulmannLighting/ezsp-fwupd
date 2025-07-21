//! A firmware update utility for devices using the `ASHv2` and `XMODEM` protocols.

pub use clear_buffer::ClearBuffer;
pub use flash_progress::FlashProgress;
pub use fwupd::{FrameCount, Fwupd, Reset};
pub use ignore_timeout::IgnoreTimeout;
pub use ota_file::OtaFile;

mod clear_buffer;
mod flash_progress;
mod fwupd;
mod ignore_timeout;
mod ota_file;
mod xmodem;
