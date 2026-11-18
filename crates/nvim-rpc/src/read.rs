//! Reads complete RPC messages into caller-provided storage.
//!
//! A receiver borrows its storage for each returned message:
//!
//! ```
//! use nvim_rpc::read::StdRead;
//! use nvim_rpc::rpc::Message;
//! use nvim_rpc::Receiver;
//!
//! let bytes = &b"\x93\x02\xa4tick\x90"[..];
//! let mut receiver = Receiver::new(StdRead(bytes), [0; 64]);
//! let Message::Notification(notification) = receiver.recv().unwrap() else {
//!     panic!("expected notification");
//! };
//! assert_eq!(notification.method.as_slice(), b"tick");
//! assert_eq!(notification.params.len(), 0);
//! ```
//!
//! Arrays, mutable slices, and pre-sized vectors can supply storage. Receiving
//! never grows it automatically.
//!
//! With the `tokio` feature, the same framing works over Tokio I/O:
//!
//! ```
//! # #[cfg(feature = "tokio")]
//! # {
//! use nvim_rpc::read::TokioRead;
//! use nvim_rpc::rpc::Message;
//! use nvim_rpc::Receiver;
//! use tokio::io::AsyncWriteExt as _;
//!
//! let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
//! runtime.block_on(async {
//!     let (mut writer, reader) = tokio::io::duplex(64);
//!     writer.write_all(b"\x93\x02\xa4tick\x90").await.unwrap();
//!     let mut receiver = Receiver::new(TokioRead(reader), [0; 64]);
//!     let Message::Notification(message) = receiver.recv_async().await.unwrap() else {
//!         panic!("expected notification");
//!     };
//!     assert_eq!(message.method.as_slice(), b"tick");
//! });
//! # }
//! ```

use crate::rpc::Message;

use mp::decode::{Object, SeekState};

use std::fmt;
use std::ops::ControlFlow;

/// Reads bytes into the supplied destination, returning zero at EOF.
pub trait ReaderSync {
    /// The underlying reader's error type.
    type Error;

    /// Fills a prefix of `buf` and returns the number of bytes written there.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;
}

/// Asynchronously reads bytes into a destination borrowed for the operation.
///
/// Implement this with an ordinary `async fn`. The trait does not require a
/// `Send` future; the executor and caller determine that requirement.
pub trait ReaderAsync {
    /// The underlying reader's error type.
    type Error;

    /// Fills a prefix of `buf`, or returns zero at EOF.
    fn read<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> impl Future<Output = Result<usize, Self::Error>> + 'a;
}

impl<R: ReaderSync + ?Sized> ReaderSync for &mut R {
    type Error = R::Error;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        R::read(self, buf)
    }
}

impl<R: ReaderAsync + ?Sized> ReaderAsync for &mut R {
    type Error = R::Error;

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        R::read(self, buf).await
    }
}

/// Adapts [`std::io::Read`], retrying interrupted reads.
#[derive(Debug)]
pub struct StdRead<R>(pub R);

impl<R: std::io::Read> ReaderSync for StdRead<R> {
    type Error = std::io::Error;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        loop {
            match self.0.read(buf) {
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                ret => return ret,
            }
        }
    }
}

#[cfg(feature = "tokio")]
/// Adapts a [`tokio::io::AsyncRead`] reader that implements [`Unpin`].
#[derive(Debug)]
pub struct TokioRead<R>(pub R);

#[cfg(feature = "tokio")]
impl<R: tokio::io::AsyncRead + Unpin> ReaderAsync for TokioRead<R> {
    type Error = std::io::Error;

    async fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Result<usize, Self::Error> {
        tokio::io::AsyncReadExt::read(&mut self.0, buf).await
    }
}

#[cfg(feature = "futures")]
/// Adapts a [`futures_io::AsyncRead`] reader that implements [`Unpin`].
#[derive(Debug)]
pub struct FuturesRead<R>(pub R);

#[cfg(feature = "futures")]
impl<R: futures_io::AsyncRead + Unpin> ReaderAsync for FuturesRead<R> {
    type Error = std::io::Error;

    async fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Result<usize, Self::Error> {
        use std::pin::Pin;
        std::future::poll_fn(|cx| Pin::new(&mut self.0).poll_read(cx, buf)).await
    }
}

/// Reading failed, the frame exceeded storage, or the input ended.
#[derive(Debug)]
pub enum ReadError<E> {
    /// The transport's error; already buffered bytes remain available.
    Read(E),
    /// More bytes are needed, but the current storage is full.
    BufferFull { capacity: usize },
    /// EOF, with the number of bytes in an unfinished frame (zero for clean EOF).
    Eof { buffered: usize },
}

impl<E: fmt::Display> fmt::Display for ReadError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(e) => write!(f, "could not read RPC message: {e}"),
            Self::BufferFull { capacity } => write!(f, "RPC frame exceeds {capacity}-byte buffer"),
            Self::Eof { buffered: 0 } => f.write_str("RPC stream ended"),
            Self::Eof { buffered } => {
                write!(f, "RPC stream ended with {buffered} incomplete bytes")
            }
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for ReadError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read(e) => Some(e),
            _ => None,
        }
    }
}

/// A reader and reusable storage for incrementally framing incoming RPC messages.
///
/// The storage's length, rather than a vector's spare capacity, limits each
/// frame. Use `vec![0; capacity]` when selecting vector-backed storage.
#[derive(Debug)]
pub struct Receiver<R, B> {
    reader: R,
    storage: B,
    start: usize,
    end: usize,
    scanned: usize,
    frame: SeekState,
}

impl<R, B> Receiver<R, B> {
    /// Uses the supplied reader and storage without allocating additional storage.
    pub fn new(reader: R, storage: B) -> Self {
        Self {
            reader,
            storage,
            start: 0,
            end: 0,
            scanned: 0,
            frame: SeekState::new(),
        }
    }

    /// Borrows the underlying reader.
    pub fn reader(&self) -> &R {
        &self.reader
    }

    /// Mutably borrows the reader; bypassing framing can consume RPC bytes.
    pub fn reader_mut(&mut self) -> &mut R {
        &mut self.reader
    }
}

impl<R, B: AsRef<[u8]>> Receiver<R, B> {
    /// Bytes retained for the next frame, including an incomplete frame.
    pub fn buffered(&self) -> &[u8] {
        &self.storage.as_ref()[self.start..self.end]
    }

    /// Recovers the reader, storage, and range containing unread bytes.
    pub fn into_parts(self) -> (R, B, std::ops::Range<usize>) {
        (self.reader, self.storage, self.start..self.end)
    }

    /// Replaces storage, copying unread bytes and retaining framing progress.
    ///
    /// Returns the original receiver if the replacement cannot hold its unread
    /// bytes. A full receiver can therefore be retried with larger storage:
    ///
    /// ```
    /// use nvim_rpc::read::StdRead;
    /// use nvim_rpc::rpc::Message;
    /// use nvim_rpc::{ReadError, Receiver};
    ///
    /// let bytes = &b"\x93\x02\xa4tick\x90"[..];
    /// let mut receiver = Receiver::new(StdRead(bytes), [0; 4]);
    /// assert!(matches!(receiver.recv(), Err(ReadError::BufferFull { capacity: 4 })));
    /// let mut receiver = receiver.with_storage([0; 32]).unwrap();
    /// let Message::Notification(message) = receiver.recv().unwrap() else {
    ///     panic!("expected notification");
    /// };
    /// assert_eq!(message.method.as_slice(), b"tick");
    /// ```
    pub fn with_storage<C: AsRef<[u8]> + AsMut<[u8]>>(
        self,
        mut storage: C,
    ) -> Result<Receiver<R, C>, Self> {
        let len = self.end - self.start;
        if storage.as_ref().len() < len {
            return Err(self);
        }
        storage.as_mut()[..len].copy_from_slice(self.buffered());
        Ok(Receiver {
            reader: self.reader,
            storage,
            start: 0,
            end: len,
            scanned: self.scanned - self.start,
            frame: self.frame,
        })
    }

    fn next_frame(&mut self) -> Option<(usize, usize)> {
        let mut bytes = &self.storage.as_ref()[self.scanned..self.end];
        let len = bytes.len();
        let ret = self.frame.transit(&mut bytes);
        self.scanned += len - bytes.len();
        if let ControlFlow::Break(_) = ret {
            let start = self.start;
            self.start = self.scanned;
            Some((start, self.scanned))
        } else {
            None
        }
    }
}

impl<R, B: AsRef<[u8]> + AsMut<[u8]>> Receiver<R, B> {
    fn compact(&mut self) {
        if self.start != 0 {
            self.storage.as_mut().copy_within(self.start..self.end, 0);
            self.end -= self.start;
            self.scanned -= self.start;
            self.start = 0;
        }
    }

    /// Reads one complete message, borrowing its payload from this receiver.
    ///
    /// A complete MessagePack value with an invalid RPC envelope is returned as
    /// [`Message::InvalidFormat`]. Partial input and framing state survive a
    /// transport error or [`ReadError::BufferFull`].
    pub fn recv<E>(&mut self) -> Result<Message<&[u8]>, ReadError<E>>
    where
        R: ReaderSync<Error = E>,
    {
        loop {
            if let Some((start, end)) = self.next_frame() {
                let bytes = &self.storage.as_ref()[start..end];
                return Ok(Object::from_buf_unchecked(bytes).into());
            }
            self.compact();
            let capacity = self.storage.as_ref().len();
            if self.end == capacity {
                return Err(ReadError::BufferFull { capacity });
            }
            let n = self
                .reader
                .read(&mut self.storage.as_mut()[self.end..])
                .map_err(ReadError::Read)?;
            if n == 0 {
                return Err(ReadError::Eof { buffered: self.end });
            }
            self.end += n;
        }
    }

    /// Asynchronously reads one message with the borrowing contract of [`Self::recv()`].
    ///
    /// Framing progress remains in the receiver across cancellation. Custom
    /// reader implementations must also preserve unread bytes when their read
    /// future is dropped if cancellation-safe receiving is required.
    pub async fn recv_async<E>(&mut self) -> Result<Message<&[u8]>, ReadError<E>>
    where
        R: ReaderAsync<Error = E>,
    {
        loop {
            if let Some((start, end)) = self.next_frame() {
                let bytes = &self.storage.as_ref()[start..end];
                return Ok(Object::from_buf_unchecked(bytes).into());
            }
            self.compact();
            let capacity = self.storage.as_ref().len();
            if self.end == capacity {
                return Err(ReadError::BufferFull { capacity });
            }
            let n = self
                .reader
                .read(&mut self.storage.as_mut()[self.end..])
                .await
                .map_err(ReadError::Read)?;
            if n == 0 {
                return Err(ReadError::Eof { buffered: self.end });
            }
            self.end += n;
        }
    }
}
