const XMODEM_BLOCK_SIZE: usize = 128;

/// Counts the standard 128-byte XMODEM blocks required for a byte sequence.
///
/// This is useful for sizing an [`indicatif::ProgressBar`] passed to [`crate::Fwupd::fwupd`].
pub trait FrameCount {
    /// Returns the number of XMODEM blocks required to transmit the referenced bytes.
    fn frame_count(&self) -> usize;
}

impl<T> FrameCount for T
where
    T: AsRef<[u8]>,
{
    fn frame_count(&self) -> usize {
        self.as_ref().len().div_ceil(XMODEM_BLOCK_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::{FrameCount, XMODEM_BLOCK_SIZE};

    #[test]
    fn counts_partial_final_block() {
        let firmware = vec![0; XMODEM_BLOCK_SIZE + 1];

        assert_eq!(firmware.frame_count(), 2);
    }
}
