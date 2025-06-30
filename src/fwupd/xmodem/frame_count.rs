use super::frame::PAYLOAD_SIZE;

pub trait FrameCount {
    /// Returns the number of frames in the XMODEM transfer.
    fn frame_count(&self) -> usize;
}

impl<T> FrameCount for T
where
    T: AsRef<[u8]>,
{
    fn frame_count(&self) -> usize {
        self.as_ref().len().div_ceil(PAYLOAD_SIZE)
    }
}
