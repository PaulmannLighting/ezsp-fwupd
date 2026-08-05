# ezsp-fwupd

`ezsp-fwupd` updates Silicon Labs Gecko-based Zigbee network co-processors that normally communicate
with a host through ASHv2 and EZSP.

The workspace provides:

- `ezsp-fwupd`: a library that enters the standalone bootloader and uploads a GBL image;
- `cli`: an interactive command-line utility for flashing, querying, and inspecting OTA files;
- `auto-updater`: a manifest-driven updater that verifies the firmware version after reboot.

## Update process

The high-level `Fwupd` implementation:

1. Creates a temporary asynchronous ASHv2/EZSP connection on the supplied native serial port.
2. Sends the EZSP command that launches the Gecko standalone bootloader.
3. Stops the protocol actors and recovers the same native serial port.
4. Disables application-level serial flow control and wakes the bootloader's ASCII menu.
5. Selects menu option `1` and transfers the GBL bytes with the external
   [`xmodem`](https://crates.io/crates/xmodem) implementation.
6. Waits for the bootloader menu to return and selects option `2` to run the application.
7. Restores the port's original timeout and flow-control settings.

The default Gecko UART bootloader uses 115,200 baud, 8 data bits, no parity, one stop bit, and no
flow control. The application firmware may use different flow control; configure the port for the
application before calling `fwupd()`. The library disables flow control only while the bootloader is
active.

## Library usage

`Fwupd::fwupd` accepts an iterator of raw GBL bytes. A Zigbee OTA container can be parsed with
`OtaFile`; its payload is the byte stream passed to the bootloader.

```rust,no_run
use std::fs::read;
use std::time::Duration;

use ezsp_fwupd::{FrameCount, Fwupd, OtaFile};
use indicatif::ProgressBar;
use le_stream::FromLeStream;
use serialport::FlowControl;

const BAUD_RATE: u32 = 115_200;

# async fn flash(path: &str) -> Result<(), Box<dyn std::error::Error>> {
let ota = OtaFile::from_le_stream_exact(read(path)?.into_iter())?
    .validate()
    .map_err(|_| std::io::Error::other("invalid OTA magic"))?;
let firmware = ota.into_payload();
let progress = ProgressBar::new(firmware.frame_count() as u64);

let serial_port = serialport::new("/dev/ttyUSB0", BAUD_RATE)
    .flow_control(FlowControl::Software)
    .open_native()?;

let _serial_port = serial_port
    .fwupd(
        firmware,
        Some(Duration::from_secs(1)),
        Some(&progress),
    )
    .await?;
# Ok(())
# }
```

The timeout applies to the synchronous Gecko menu and XMODEM stages. Supplying `None` keeps the
port's existing timeout. The optional progress bar advances once per standard 128-byte XMODEM block.
On success, the returned value is the original serial port with its previous settings restored.

For direct EZSP access, use `make_uart(...).await`. It returns an `ezsp::Connection` and a `Tasks`
owner. Keep `Tasks` alive while using the connection; `Tasks::terminate().await` stops and joins the
protocol actors and returns the underlying serial port.

`GeckoBootloader` exposes the lower-level ASCII menu operations. After
`start_xmodem_upload()`, leave the receiver's initial ASCII `C` unread because `xmodem::Xmodem::send`
uses it to negotiate CRC mode.

## Command-line utility

Run the CLI from the workspace with:

```console
cargo run -p cli -- flash /dev/ttyUSB0 firmware.ota
cargo run -p cli -- query /dev/ttyUSB0
cargo run -p cli -- reset /dev/ttyUSB0
cargo run -p cli -- ota firmware.ota
```

Use `--help` on the program or a subcommand for timeout and diagnostic options.

## Automatic updater

The automatic updater reads a JSON manifest, compares its version with the running firmware,
performs an upgrade or downgrade when required, waits for reboot, and validates the resulting
version. Its default manifest path is `/etc/ezsp-firmware-update.json`.

```json
{
  "active": {
    "version": "1.2.3",
    "filename": "/var/lib/ezsp/firmware-1.2.3.ota"
  }
}
```

Run it with:

```console
cargo run -p auto-updater -- /dev/ttyUSB0
```

## Limitations

- The project is under active development and its API may change.
- The implementation targets interactive Gecko standalone UART bootloaders and standard 128-byte
  XMODEM blocks.
- `xmodem` 0.4 counts unexpected block responses against its error budget but does not replay the
  same input block. A noisy serial link can therefore abort an update rather than recover through a
  retransmission.
