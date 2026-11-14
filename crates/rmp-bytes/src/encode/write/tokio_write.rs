use super::WriterAsync;

use tokio::io::{AsyncWrite, AsyncWriteExt};

/// Adapts a Tokio [`AsyncWrite`] destination to [`WriterAsync`].
///
/// ```
/// use rmp_bytes::encode::{TokioWrite, WriterAsync as _, WriterAsyncExt as _};
/// use tokio::io::AsyncWrite;
///
/// async fn write_message<W: AsyncWrite + Unpin>(output: W) -> std::io::Result<()> {
///     let mut writer = TokioWrite::new(output);
///     writer.write_primitive(("event", 7u8)).consume_async().await?;
///     writer.flush().await
/// }
/// ```
pub struct TokioWrite<W> {
    inner: W,
}

impl<W> TokioWrite<W> {
    /// Wraps a Tokio writer. The adapter does not start a task.
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

impl<W: AsyncWrite + Unpin> WriterAsync for TokioWrite<W> {
    type Result = std::io::Result<()>;

    async fn write_all(&mut self, buf: &[u8]) -> Self::Result {
        AsyncWriteExt::write_all(&mut self.inner, buf).await
    }

    async fn flush(&mut self) -> Self::Result {
        AsyncWriteExt::flush(&mut self.inner).await
    }
}
