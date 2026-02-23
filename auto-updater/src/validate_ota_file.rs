use std::fs::read;

use ezsp_fwupd::OtaFile;
use le_stream::FromLeStream;
use log::error;

use crate::manifest::Metadata;

/// Validate the OTA file by reading it and checking its contents.
pub fn validate_ota_file(metadata: &Metadata) -> Option<OtaFile> {
    let ota_file_bytes = read(metadata.filename())
        .inspect_err(|error| error!("Failed to read firmware file: {error}"))
        .ok()?;

    let ota_file = OtaFile::from_le_stream_exact(ota_file_bytes.into_iter())
        .inspect_err(|error| error!("Failed to parse OTA file: {error}"))
        .ok()?;

    ota_file.header().log();
    ota_file
        .validate()
        .inspect_err(|error| {
            error!("Invalid OTA file magic: {error:#04X?}");
        })
        .ok()
}
