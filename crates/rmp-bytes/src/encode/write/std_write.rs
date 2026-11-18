use super::WriterSync;

use std::io::Result;
use std::io::Write;

/// Adapts a [`std::io::Write`] destination to [`WriterSync`].
///
/// The adapter calls [`std::io::Write::write_all()`] for each encoding chunk.
/// Use [`WriterSync::flush()`] when the destination's buffering or transport
/// requires it.
pub struct StdWrite<W> {
    inner: W,
}

impl<W> StdWrite<W> {
    /// Wraps a standard I/O writer.
    pub fn new(wtr: W) -> Self {
        Self { inner: wtr }
    }

    /// Borrows the underlying writer.
    pub fn get_ref(&self) -> &W {
        &self.inner
    }

    /// Mutably borrows the underlying writer.
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    /// Returns the underlying writer.
    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: Write> WriterSync for StdWrite<W> {
    type Result = Result<()>;

    fn write_all(&mut self, buf: &[u8]) -> Self::Result {
        self.inner.write_all(buf)
    }

    fn flush(&mut self) -> Self::Result {
        Write::flush(&mut self.inner)
    }
}
