//! MessagePack encoding and decoding with caller-selected buffers and I/O.
//!
//! Encode a tuple directly into a fixed-size output buffer:
//!
//! ```
//! use rmp_bytes::encode::{StdWrite, WriterSyncExt as _};
//! use std::io::Cursor;
//!
//! let mut storage = [0; 32];
//! let mut cursor = Cursor::new(storage.as_mut_slice());
//! StdWrite::new(&mut cursor)
//!     .write_primitive(("hello", 42u8))
//!     .consume()?;
//! let len = cursor.position() as usize;
//! assert_eq!(&cursor.get_ref()[..len], b"\x92\xa5hello\x2a");
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! Decode an array without building a collection of its elements:
//!
//! ```
//! use rmp_bytes::decode::{Object, ObjectParser};
//!
//! let bytes = b"\x92\xa5hello\x2a";
//! let Object::Array(mut values) = ObjectParser::new(bytes.as_slice()).next_object()?
//! else { panic!("expected an array") };
//! let Some(Object::Str(text)) = values.next() else { panic!("expected a string") };
//! assert_eq!(text.as_str()?, "hello");
//! assert_eq!(values.next().unwrap().to_u8(), Some(42));
//! assert!(values.next().is_none());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! [`encode::primitive`] provides scalar encoders and array/map adapters.
//! [`decode::ObjectParser`] reads complete values from a buffer;
//! [`decode::SeekState`] can locate their boundaries across input chunks.
//! Writers send encoded chunks to a `Vec`, a standard I/O writer, or an optional
//! async adapter. The crate does not start tasks or select an executor.
//!
//! # Features
//!
//! There are no default features. Enable `bytes` for `bytes` buffer support,
//! `tokio-write` or `futures-write` for their respective writer adapters, and
//! `rpc` for MessagePack-RPC request, response, and notification types.

pub mod decode;
pub mod encode;

#[cfg(feature = "bytes")]
pub use bytes;
pub use rmp;

#[cfg(feature = "rpc")]
pub mod rpc;

/// The type ID of MessagePack's `Extension` type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExtType(pub i8);

impl ExtType {
    pub const TIMESTAMP: Self = Self(-1);
}
