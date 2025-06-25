use crate::silabs2::xmodem::{PAYLOAD_SIZE, Payload, Xmodem};

/// An iterator that produces Xmodem frames from a byte stream.
#[derive(Debug)]
pub struct XmodemFrames<T> {
    bytes: T,
    index: u8,
}

impl<T> XmodemFrames<T> {
    /// Creates a new `XmodemFrames` iterator from the given byte stream.
    pub const fn new(bytes: T) -> Self {
        Self { bytes, index: 0 }
    }
}

impl<T> Iterator for XmodemFrames<T>
where
    T: Iterator<Item = u8>,
{
    type Item = Xmodem;

    fn next(&mut self) -> Option<Self::Item> {
        let payload: Payload = (&mut self.bytes).take(PAYLOAD_SIZE).collect();

        if payload.is_empty() {
            None
        } else {
            let frame = Xmodem::new(self.index, payload);
            self.index = self.index.wrapping_add(1);
            Some(frame)
        }
    }
}

impl<T> From<T> for XmodemFrames<T::IntoIter>
where
    T: IntoIterator<Item = u8>,
{
    fn from(bytes: T) -> Self {
        Self::new(bytes.into_iter())
    }
}
