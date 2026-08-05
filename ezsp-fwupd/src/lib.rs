//! A firmware update utility for devices using the `ASHv2` and `XMODEM` protocols.

pub use self::clear_buffer::ClearBuffer;
pub use self::discard_callbacks::discard_callbacks;
pub use self::flash_progress::FlashProgress;
pub use self::frame_count::FrameCount;
pub use self::fwupd::{Fwupd, GeckoBootloader};
pub use self::ignore_timeout::IgnoreTimeout;
pub use self::make_uart::{MakeUartError, Tasks, make_uart};
pub use self::ota_file::OtaFile;

mod clear_buffer;
mod discard_callbacks;
mod flash_progress;
mod frame_count;
mod fwupd;
mod ignore_timeout;
mod launch_bootloader;
mod make_uart;
mod ota_file;
