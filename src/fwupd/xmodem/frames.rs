use super::frame::{Frame, PAYLOAD_SIZE, Payload};
use crate::fill::Fill;

const FILLER: u8 = 0xFF;

/// An iterator that produces Xmodem frames from a byte stream.
#[derive(Debug)]
pub struct Frames<T> {
    bytes: T,
    buffer: Payload,
    index: u8,
}

impl<T> Frames<T> {
    /// Creates a new `XmodemFrames` iterator from the given byte stream.
    pub const fn new(bytes: T) -> Self {
        Self {
            bytes,
            buffer: [0; PAYLOAD_SIZE],
            index: 1,
        }
    }
}

impl<T> Iterator for Frames<T>
where
    T: Iterator<Item = u8>,
{
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        for (dst, src) in self
            .buffer
            .iter_mut()
            .zip(self.bytes.by_ref().take(PAYLOAD_SIZE).fill(FILLER))
        {
            *dst = src;
        }

        if self.buffer.is_empty() {
            None
        } else {
            let frame = Frame::new(self.index, self.buffer);
            self.index = self.index.wrapping_add(1);
            Some(frame)
        }
    }
}

impl<T> From<T> for Frames<T::IntoIter>
where
    T: IntoIterator<Item = u8>,
{
    fn from(bytes: T) -> Self {
        Self::new(bytes.into_iter())
    }
}
