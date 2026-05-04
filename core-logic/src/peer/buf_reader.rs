//! Implementation for the BufRead trait for the peer connection.

use ::core::ops::Index;

/// simple buffer receiver for the peer connection,
/// it holds a buffer and the length of the data currently in the buffer.
/// used to construct a message from incoming data,
/// which might not be complete
///
/// [0 1 2 3 4 5 0 0 0 ... 0 0 0 0] len = 6, capacity = e.g. 20
///
/// - `remaining_mut`: returns a mutable slice from current length (here: 6) to end (here: 20)
/// - `advance_n(n)`: called after writing n bytes to the buffer, increases the length to n
/// - `as_slice`: returns a slice of the buffer from 0 to current length (here: 6)
pub(crate) struct BufReader<const CAP: usize> {
    buf: [u8; CAP],
    len: usize,
}

impl<const CAP: usize> BufReader<CAP> {
    pub(crate) const fn new() -> Self {
        Self {
            buf: [0; CAP],
            len: 0,
        }
    }

    pub(crate) const fn len(&self) -> usize {
        self.len
    }

    pub(crate) const fn capacity(&self) -> usize {
        CAP
    }

    pub(crate) fn advance_n(&mut self, n: usize) {
        let remaining = self.capacity() - self.len();
        if n > remaining {
            defmt_or_log::panic!("Cannot advance more than the length of the buffer");
        }
        self.len += n;
    }

    pub(crate) fn remaining_mut(&mut self) -> &mut [u8] {
        &mut self.buf[self.len..]
    }

    /// sets the length back to 0 to be written to again, does not clear the buffer
    pub(crate) const fn reset(&mut self) {
        self.len = 0;
    }

    pub(crate) fn as_slice(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

impl<const CAP: usize> AsRef<[u8]> for BufReader<CAP> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

impl<const CAP: usize> AsMut<[u8]> for BufReader<CAP> {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.buf[..self.len]
    }
}

impl<const CAP: usize> Index<usize> for BufReader<CAP> {
    type Output = u8;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len {
            defmt_or_log::panic!("Index out of bounds for BufReader");
        }
        &self.buf[index]
    }
}

#[cfg(test)]
mod tests {
    use super::BufReader;

    #[test]
    fn test_buf_reader() {
        let mut buf = BufReader::<10>::new();
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.capacity(), 10);

        buf.remaining_mut()[..5].copy_from_slice(&[1, 2, 3, 4, 5]);
        buf.advance_n(5);

        assert_eq!(buf.as_ref(), &[1, 2, 3, 4, 5]);

        buf.remaining_mut()[..2].copy_from_slice(&[6, 7]);
        buf.advance_n(2);

        assert_eq!(buf.as_ref(), &[1, 2, 3, 4, 5, 6, 7]);

        buf.reset();

        assert_eq!(buf.len(), 0);
    }
}
