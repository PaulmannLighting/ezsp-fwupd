//! A firmware update utility for devices using the `ASHv2` and `XMODEM` protocols.

pub use fwupd::{FrameCount, Fwupd, Reset, Tty};
pub use ota_file::OtaFile;

mod clear_buffer;
mod fwupd;
mod ignore_timeout;
mod ota_file;
mod xmodem;
