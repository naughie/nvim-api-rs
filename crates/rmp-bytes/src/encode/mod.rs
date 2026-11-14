//! Encode values as a sequence of byte chunks.
//!
//! A [`Primitive`] creates its encoding state; a writer consumes that state.
//! A `Vec<u8>` is already a writer, while [`StdWrite`] adapts `std::io::Write`:
//!
//! ```
//! use rmp_bytes::encode::WriterSyncExt as _;
//! use rmp_bytes::encode::primitive::{Array, Map};
//!
//! let mut bytes = Vec::new();
//! bytes.write_primitive(Map([("enabled", true)])).consume();
//! assert_eq!(bytes, b"\x81\xa7enabled\xc3");
//!
//! bytes.clear();
//! let words = ["red", "blue"];
//! bytes.write_primitive(Array(&words[..])).consume();
//! assert_eq!(bytes, b"\x92\xa3red\xa4blue");
//! ```
//!
//! [`WriterSyncExt::write_primitive`] and [`WriterAsyncExt::write_primitive`]
//! prepare a write. Call `consume()` or await `consume_async()` to finish it.
//! Writers preserve their own result type, including `()` for infallible buffers.

pub mod write;
pub use write::StdWrite;
pub use write::{WriterAsync, WriterAsyncExt, WriterSync, WriterSyncExt};

#[cfg(feature = "bytes")]
pub use write::BufWriter;
#[cfg(feature = "futures-write")]
pub use write::FuturesWrite;
#[cfg(feature = "tokio-write")]
pub use write::TokioWrite;

pub mod primitive;
pub use primitive::Primitive;

#[cfg(feature = "rpc")]
pub mod rpc;
