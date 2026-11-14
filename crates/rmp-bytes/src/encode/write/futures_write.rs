use super::WriterAsync;

use futures_util::io::{AsyncWrite, AsyncWriteExt};

/// Adapts a futures [`AsyncWrite`] destination to [`WriterAsync`].
///
/// The adapter borrows chunks across `write_all().await` and does not start a
/// task. Use [`WriterAsync::flush`] to flush the underlying writer explicitly.
pub struct FuturesWrite<W> {
    inner: W,
}

impl<W> FuturesWrite<W> {
    /// Wraps a futures I/O writer.
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

impl<W: AsyncWrite + Unpin> WriterAsync for FuturesWrite<W> {
    type Result = std::io::Result<()>;

    async fn write_all(&mut self, buf: &[u8]) -> Self::Result {
        AsyncWriteExt::write_all(&mut self.inner, buf).await
    }

    async fn flush(&mut self) -> Self::Result {
        AsyncWriteExt::flush(&mut self.inner).await
    }
}
