//! Firmware updates for EZSP-based Zigbee network co-processors.
//!
//! The crate uses `ASHv2` and EZSP to launch a Silicon Labs Gecko standalone bootloader, takes back
//! ownership of the native serial port, and uploads a GBL image with XMODEM-CRC. The high-level
//! [`Fwupd`] trait coordinates that complete transition and returns the same serial port after the
//! bootloader has been asked to run the application image.
//!
//! [`GeckoBootloader`] exposes the interactive bootloader menu operations for lower-level use.
//! [`make_uart`] and [`Tasks`] expose the ASHv2/EZSP connection lifecycle independently.

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
