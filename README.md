# ezsp-fwupd

A Firmware updater for [`EZSP`](https://github.com/PaulmannLighting/ezsp)-based Zigbee WNICs.

This library provides a simple interface to update the firmware of devices that use the EZSP protocol.

## Library API

Firmware updates operate on a native [`serialport`](https://crates.io/crates/serialport) port. The
library temporarily runs the asynchronous ASHv2 and EZSP actor stacks to launch the standalone
bootloader, then returns the same serial port and transfers the firmware with XMODEM.

```rust,no_run
use std::time::Duration;

use ezsp_fwupd::Fwupd;
use serialport::FlowControl;

const BAUD_RATE: u32 = 115_200;

# async fn flash(firmware: Vec<u8>) -> std::io::Result<()> {
let serial_port = serialport::new("/dev/ttyUSB0", BAUD_RATE)
    .flow_control(FlowControl::Software)
    .open_native()?;

serial_port
    .fwupd(firmware, Some(Duration::from_secs(1)), None)
    .await?;
# Ok(())
# }
```

Use `make_uart(...).await` when direct EZSP access is required. It returns an `ezsp::Connection`
and a `Tasks` owner. Calling `Tasks::terminate().await` stops the actor stack and returns the
underlying serial port.

## In development

This library is currently under heavy development and is not yet ready for production use. The API may change
frequently, and there may be bugs and missing features.
