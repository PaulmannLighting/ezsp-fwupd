//! A firmware update utility for devices using the `ASHv2` and `XMODEM` protocols.

pub use self::clear_buffer::ClearBuffer;
pub use self::discard_callbacks::discard_callbacks;
pub use self::flash_progress::FlashProgress;
pub use self::fwupd::{FrameCount, Fwupd, GeckoBootloader};
pub use self::ignore_timeout::IgnoreTimeout;
pub use self::make_uart::{MakeUartError, Tasks, make_uart};
pub use self::ota_file::OtaFile;

mod clear_buffer;
mod discard_callbacks;
mod flash_progress;
mod fwupd;
mod ignore_timeout;
mod launch_bootloader;
mod make_uart;
mod ota_file;
mod xmodem;
