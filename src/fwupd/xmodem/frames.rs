use super::frame::{Frame, PAYLOAD_SIZE, Payload};

/// An iterator that produces Xmodem frames from a byte stream.
#[derive(Debug)]
pub struct Frames<T> {
    bytes: T,
    index: u8,
}

impl<T> Frames<T> {
    /// Creates a new `XmodemFrames` iterator from the given byte stream.
    pub const fn new(bytes: T) -> Self {
        Self { bytes, index: 1 }
    }
}

impl<T> Iterator for Frames<T>
where
    T: Iterator<Item = u8>,
{
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        let payload: Payload = (&mut self.bytes).take(PAYLOAD_SIZE).collect();

        if payload.is_empty() {
            None
        } else {
            let frame = Frame::new(self.index, payload);
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
