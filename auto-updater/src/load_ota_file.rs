use std::fs::read;

use ezsp_fwupd::OtaFile;
use le_stream::FromLeStream;
use log::error;

use crate::manifest::Metadata;

/// Extension trait to load and validate the OTA file from the metadata.
pub trait LoadOtaFile {
    /// Load and validate the OTA file by reading it and checking its contents.
    fn load_ota_file(&self) -> Option<OtaFile>;
}

impl LoadOtaFile for Metadata {
    fn load_ota_file(&self) -> Option<OtaFile> {
        let ota_file_bytes = read(self.filename())
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
}
