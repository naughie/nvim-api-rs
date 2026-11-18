use crate::mp::decode::SeekState;
use crate::{Bytes, ConnectionError};

use bytes::BytesMut;
use tokio::io::{AsyncRead, AsyncReadExt as _};

use std::ops::ControlFlow;

pub(crate) struct Frames<R> {
    reader: R,
    buf: BytesMut,
    scanned: usize,
    state: SeekState,
    max_bytes: usize,
}

impl<R> Frames<R> {
    pub(crate) fn new(reader: R, max_bytes: usize) -> Self {
        Self {
            reader,
            buf: BytesMut::new(),
            scanned: 0,
            state: SeekState::new(),
            max_bytes,
        }
    }

    pub(crate) fn buffered(&mut self) -> Option<Bytes> {
        let mut cursor = self.buf.get(self.scanned..)?;
        match self.state.transit(&mut cursor) {
            ControlFlow::Break(n) => {
                // transit consumes at most buf.len() - scanned bytes, so
                // scanned + n <= buf.len() <= usize::MAX and split_to is valid.
                let len = self.scanned + n.get();
                self.scanned = 0;
                Some(self.buf.split_to(len).freeze())
            }
            ControlFlow::Continue(()) => {
                self.scanned = self.buf.len();
                None
            }
        }
    }
}

impl<R: AsyncRead + Unpin> Frames<R> {
    pub(crate) async fn next(&mut self) -> Result<Option<Bytes>, ConnectionError> {
        loop {
            if let Some(frame) = self.buffered() {
                return Ok(Some(frame));
            }

            if self.buf.len() >= self.max_bytes {
                return Err(ConnectionError::Limit("RPC frame bytes"));
            }
            // The comparison above proves len < max_bytes, so subtraction
            // cannot underflow; limiting each read also bounds buffer growth.
            let remaining = (self.max_bytes - self.buf.len()).min(8192) as u64;
            let n = (&mut self.reader)
                .take(remaining)
                .read_buf(&mut self.buf)
                .await?;
            if n == 0 {
                return if self.buf.is_empty() {
                    Ok(None)
                } else {
                    Err(ConnectionError::Eof {
                        buffered: self.buf.len(),
                    })
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tokio::io::ReadBuf;

    use std::io;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    struct Chunked(&'static [u8]);

    impl AsyncRead for Chunked {
        fn poll_read(
            mut self: Pin<&mut Self>,
            _: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            let n = self.0.len().min(buf.remaining()).min(1);
            buf.put_slice(&self.0[..n]);
            self.0 = &self.0[n..];
            Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn fragments_nested_values_and_retains_following_frames() {
        let bytes = b"\x92\xdb\x00\x00\x00\x03abc\xc7\x02\x05xy\xc3";
        let mut frames = Frames::new(Chunked(bytes), usize::MAX);
        assert_eq!(frames.next().await.unwrap().unwrap(), &bytes[..14]);
        assert_eq!(frames.next().await.unwrap().unwrap(), &b"\xc3"[..]);
        assert!(frames.next().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn separates_coalesced_frames() {
        let mut frames = Frames::new(&b"\xc0\xc3\x91\x01"[..], usize::MAX);
        for bytes in [&b"\xc0"[..], &b"\xc3"[..], &b"\x91\x01"[..]] {
            assert_eq!(frames.next().await.unwrap().unwrap(), bytes);
        }
        assert!(frames.next().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn reports_incomplete_bytes_after_complete_frames() {
        let mut frames = Frames::new(&b"\xc0\xda\x00"[..], usize::MAX);
        assert_eq!(frames.next().await.unwrap().unwrap(), &b"\xc0"[..]);
        assert!(matches!(
            frames.next().await,
            Err(ConnectionError::Eof { buffered: 2 })
        ));
    }

    #[tokio::test]
    async fn preserves_native_io_errors() {
        struct Fails;
        impl AsyncRead for Fails {
            fn poll_read(
                self: Pin<&mut Self>,
                _: &mut Context<'_>,
                _: &mut ReadBuf<'_>,
            ) -> Poll<io::Result<()>> {
                Poll::Ready(Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "test reader",
                )))
            }
        }

        let Err(ConnectionError::Io(e)) = Frames::new(Fails, usize::MAX).next().await else {
            panic!("expected the reader's error");
        };
        assert_eq!(e.kind(), io::ErrorKind::BrokenPipe);
        assert_eq!(e.to_string(), "test reader");
    }
}
