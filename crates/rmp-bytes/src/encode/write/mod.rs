//! Send primitive encodings to synchronous or asynchronous writers.
//!
//! `Vec<u8>` implements both writer traits with result `()`. [`StdWrite`] uses
//! `std::io::Result<()>`; the optional Tokio and futures adapters use the same
//! result type. Adapter writes do not flush the underlying transport.
//!
//! A custom async writer can implement the trait with an ordinary `async fn`:
//!
//! ```
//! use rmp_bytes::encode::{WriterAsync, WriterAsyncExt as _};
//!
//! struct Output(Vec<u8>);
//!
//! impl WriterAsync for Output {
//!     type Result = ();
//!
//!     async fn write_all(&mut self, buf: &[u8]) {
//!         self.0.extend_from_slice(buf);
//!     }
//!
//!     async fn flush(&mut self) {}
//! }
//!
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! let mut output = Output(Vec::new());
//! output.write_primitive((true, 7u8)).consume_async().await;
//! assert_eq!(output.0, b"\x92\xc3\x07");
//! # });
//! ```
//!
//! Neither trait requires `Send`, and an async implementation's future need not
//! be `Send`. Choose the executor and any task spawning at the application layer.

use super::Encode;
use super::primitive::{Primitive, State};

use std::ops::ControlFlow;

mod buffer_write;
#[cfg(feature = "bytes")]
pub use buffer_write::BufWriter;

mod std_write;
pub use std_write::StdWrite;

#[cfg(feature = "tokio-write")]
mod tokio_write;
#[cfg(feature = "tokio-write")]
pub use tokio_write::TokioWrite;

#[cfg(feature = "futures-write")]
mod futures_write;
#[cfg(feature = "futures-write")]
pub use futures_write::FuturesWrite;

#[cfg(test)]
mod tests;

/// Writes every byte of each supplied chunk before returning success.
pub trait WriterSync {
    /// The implementation's success/failure representation.
    type Result: WriteResult;

    /// Writes the entire chunk, or returns a failure result.
    fn write_all(&mut self, buf: &[u8]) -> Self::Result;

    /// Flushes buffered bytes to the destination, or succeeds immediately for memory buffers.
    fn flush(&mut self) -> Self::Result;
}

/// Writes complete chunks asynchronously, borrowing the writer and chunk.
///
/// Implementations can use `async fn write_all(&mut self, buf: &[u8])`.
pub trait WriterAsync {
    /// The implementation's success/failure representation.
    type Result: WriteResult;

    /// Writes the entire chunk without requiring a named future type.
    fn write_all<'a>(&'a mut self, buf: &'a [u8]) -> impl Future<Output = Self::Result> + 'a;

    /// Flushes buffered bytes asynchronously, or succeeds immediately for memory buffers.
    fn flush(&mut self) -> impl Future<Output = Self::Result> + '_;
}

/// Identifies success so an encoder can stop on the first failed chunk.
///
/// Implemented for `()`, `Result<(), E>`, `Option<()>`, and `ControlFlow<E>`.
pub trait WriteResult {
    /// Constructs the result of a completed successful write.
    fn ok() -> Self;
    /// Reports whether encoding may continue.
    fn is_ok(&self) -> bool;
}

/// Runs the next write only if the previous result succeeded.
pub trait WriteResultExt: WriteResult {
    /// Calls `then` on success; otherwise preserves the current result.
    fn and_then<F: FnOnce() -> Self>(self, then: F) -> Self;

    /// Awaits `then` on success; otherwise preserves the current result.
    fn and_then_async<F: AsyncFnOnce() -> Self>(self, then: F) -> impl Future<Output = Self>;
}

impl<W: WriteResult> WriteResultExt for W {
    fn and_then<F: FnOnce() -> Self>(self, then: F) -> Self {
        if self.is_ok() { then() } else { self }
    }

    async fn and_then_async<F: AsyncFnOnce() -> Self>(self, then: F) -> Self {
        if self.is_ok() { then().await } else { self }
    }
}

impl WriteResult for () {
    fn ok() -> Self {}
    fn is_ok(&self) -> bool {
        true
    }
}

impl<E> WriteResult for Result<(), E> {
    fn ok() -> Self {
        Ok(())
    }
    fn is_ok(&self) -> bool {
        self.is_ok()
    }
}

impl<B> WriteResult for ControlFlow<B> {
    fn ok() -> Self {
        ControlFlow::Continue(())
    }
    fn is_ok(&self) -> bool {
        self.is_continue()
    }
}

impl WriteResult for Option<()> {
    fn ok() -> Self {
        Some(())
    }
    fn is_ok(&self) -> bool {
        self.is_some()
    }
}

impl<W: WriterSync> WriterSync for &mut W {
    type Result = W::Result;

    fn write_all(&mut self, buf: &[u8]) -> Self::Result {
        W::write_all(self, buf)
    }

    fn flush(&mut self) -> Self::Result {
        W::flush(self)
    }
}

/// Adds value encoding to a synchronous writer.
pub trait WriterSyncExt: WriterSync {
    /// Creates a write operation; call [`WriteValue::consume()`] to finish it.
    fn write_primitive<P: Primitive>(&mut self, value: P) -> WritePrimitive<&mut Self, P::State>;

    /// Creates a write operation for a primitive or composed encoding.
    ///
    /// Call [`WriteValue::consume()`] to finish it.
    fn write_value<V: Encode>(&mut self, value: V) -> WriteValue<&mut Self, V::Encoder>;
}

impl<W: WriterSync> WriterSyncExt for W {
    fn write_primitive<P: Primitive>(&mut self, value: P) -> WritePrimitive<&mut Self, P::State> {
        self.write_value(value)
    }

    fn write_value<V: Encode>(&mut self, value: V) -> WriteValue<&mut Self, V::Encoder> {
        WriteValue(self, value.into_encoder(), None)
    }
}

impl<W: WriterAsync> WriterAsync for &mut W {
    type Result = W::Result;

    async fn write_all(&mut self, buf: &[u8]) -> Self::Result {
        W::write_all(self, buf).await
    }

    async fn flush(&mut self) -> Self::Result {
        W::flush(self).await
    }
}

/// Adds value encoding to an asynchronous writer.
pub trait WriterAsyncExt: WriterAsync {
    /// Creates a write operation; await [`WriteValue::consume_async()`] to finish it.
    fn write_primitive<P: Primitive>(&mut self, value: P) -> WritePrimitive<&mut Self, P::State>;

    /// Creates a write operation for a primitive or composed encoding.
    ///
    /// Await [`WriteValue::consume_async()`] to finish it.
    fn write_value<V: Encode>(&mut self, value: V) -> WriteValue<&mut Self, V::Encoder>;
}

impl<W: WriterAsync> WriterAsyncExt for W {
    fn write_primitive<P: Primitive>(&mut self, value: P) -> WritePrimitive<&mut Self, P::State> {
        self.write_value(value)
    }

    fn write_value<V: Encode>(&mut self, value: V) -> WriteValue<&mut Self, V::Encoder> {
        WriteValue(self, value.into_encoder(), None)
    }
}

/// An encoding state paired with its destination writer.
///
/// The current chunk is retained while an async write borrows its bytes. A failed
/// or canceled write may have written a prefix; restarting it does not roll back
/// the destination. Keep message-level retry policy outside this operation.
pub struct WriteValue<W, S: State>(W, S, Option<S::Item>);

/// The original name for [`WriteValue`], retained for primitive write operations.
pub type WritePrimitive<W, S> = WriteValue<W, S>;

impl<W: WriterSync, S: State> WriteValue<W, S> {
    /// Writes the next chunk, or returns `None` when all chunks are consumed.
    pub fn next_chunk(&mut self) -> Option<W::Result> {
        self.2 = self.1.next();
        Some(self.0.write_all(self.2.as_ref()?.as_ref()))
    }

    /// Writes all remaining chunks, returning the first failure.
    pub fn consume(mut self) -> W::Result {
        while let Some(ret) = self.next_chunk() {
            if !ret.is_ok() {
                return ret;
            }
        }
        <W::Result as WriteResult>::ok()
    }
}

impl<W: WriterAsync, S: State> WriteValue<W, S> {
    /// Writes one chunk asynchronously, retaining it across the await.
    pub async fn next_chunk_async(&mut self) -> Option<W::Result> {
        self.2 = self.1.next();
        let chunk = self.2.as_ref()?;
        Some(self.0.write_all(chunk.as_ref()).await)
    }

    /// Writes all remaining chunks asynchronously, returning the first failure.
    pub async fn consume_async(mut self) -> W::Result {
        while let Some(ret) = self.next_chunk_async().await {
            if !ret.is_ok() {
                return ret;
            }
        }
        <W::Result as WriteResult>::ok()
    }
}
