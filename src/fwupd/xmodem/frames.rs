use std::iter::repeat;

use super::frame::{Frame, PAYLOAD_SIZE};

const FILLER: u8 = 0xFF;

/// An iterator that produces Xmodem frames from a byte stream.
#[derive(Debug)]
pub struct Frames<T> {
    bytes: T,
    buffer: Vec<u8>,
    index: u8,
}

impl<T> Frames<T> {
    /// Creates a new `XmodemFrames` iterator from the given byte stream.
    pub const fn new(bytes: T) -> Self {
        Self {
            bytes,
            buffer: Vec::new(),
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
        self.buffer.clear();
        self.buffer.extend(self.bytes.by_ref().take(PAYLOAD_SIZE));

        if self.buffer.is_empty() {
            return None;
        }

        let mut payload = [0; PAYLOAD_SIZE];

        for (dst, src) in payload
            .iter_mut()
            .zip(self.buffer.iter().copied().chain(repeat(FILLER)))
        {
            *dst = src;
        }

        let frame = Frame::new(self.index, payload);
        self.index = self.index.wrapping_add(1);
        Some(frame)
    }
}

impl<T> ExactSizeIterator for Frames<T>
where
    T: Iterator<Item = u8> + ExactSizeIterator,
{
    fn len(&self) -> usize {
        self.bytes.len().div_ceil(PAYLOAD_SIZE)
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
