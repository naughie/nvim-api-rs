//! Read MessagePack values while retaining their buffer representation.
//!
//! [`ObjectParser`] advances over one complete value at a time. String and binary
//! payloads retain the backing buffer; array and map elements are parsed as their
//! iterators advance.
//!
//! ```
//! use rmp_bytes::decode::{Object, ObjectParser};
//!
//! let mut parser = ObjectParser::new(&b"\x81\xa1x\x2a\xc3"[..]);
//! let Object::Map(mut map) = parser.next_object()? else { panic!("expected a map") };
//! let entry = map.next().unwrap();
//! assert_eq!(entry.key.to_str().unwrap().as_str()?, "x");
//! assert_eq!(entry.value.to_u8(), Some(42));
//! assert!(map.next().is_none());
//! assert_eq!(parser.next_object()?.to_bool(), Some(true));
//! assert!(parser.remaining().is_empty());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Scalar variants reflect the decoded wire format. For example, `42` in a
//! positive fixint is [`Object::U8`], even if its producer started with a `u64`.
//! The `to_*` accessors match a variant exactly; they do not perform numeric casts.
//! [`InsufficientBytes`] means a complete value is not available. When receiving
//! fragments, retain the input and use [`SeekState`] to track the frame boundary.

mod seek;
pub use seek::{State as SeekState, seek_to_eoo};

pub mod buf;
pub use buf::{Buf, Seekable};

mod parse;
pub use parse::*;

#[cfg(feature = "rpc")]
pub mod rpc;

#[cfg(test)]
mod tests;

use std::fmt;

/// The input ended before a complete value could be decoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InsufficientBytes;

impl fmt::Display for InsufficientBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Reached EoF unexpectedly while decoding MessagePack objects"
        )
    }
}
impl std::error::Error for InsufficientBytes {}

mod read {
    use super::InsufficientBytes;
    use super::buf::Buf;

    use rmp::Marker;

    type Result<T> = std::result::Result<T, InsufficientBytes>;

    pub(super) fn marker(cursor: &mut impl Buf) -> Result<Marker> {
        self::u8(cursor).map(Marker::from_u8)
    }

    pub(super) fn u8(cursor: &mut impl Buf) -> Result<u8> {
        cursor.first_byte().ok_or(InsufficientBytes)
    }

    pub(super) fn u16(cursor: &mut impl Buf) -> Result<u16> {
        bytes_fixed::<_>(cursor).map(u16::from_be_bytes)
    }

    pub(super) fn u32(cursor: &mut impl Buf) -> Result<u32> {
        bytes_fixed::<_>(cursor).map(u32::from_be_bytes)
    }

    pub(super) fn u64(cursor: &mut impl Buf) -> Result<u64> {
        bytes_fixed::<_>(cursor).map(u64::from_be_bytes)
    }

    pub(super) fn i8(cursor: &mut impl Buf) -> Result<i8> {
        self::u8(cursor).map(|byte| byte as i8)
    }

    pub(super) fn i16(cursor: &mut impl Buf) -> Result<i16> {
        bytes_fixed::<_>(cursor).map(i16::from_be_bytes)
    }

    pub(super) fn i32(cursor: &mut impl Buf) -> Result<i32> {
        bytes_fixed::<_>(cursor).map(i32::from_be_bytes)
    }

    pub(super) fn i64(cursor: &mut impl Buf) -> Result<i64> {
        bytes_fixed::<_>(cursor).map(i64::from_be_bytes)
    }

    pub(super) fn f32(cursor: &mut impl Buf) -> Result<f32> {
        bytes_fixed::<_>(cursor).map(f32::from_be_bytes)
    }

    pub(super) fn f64(cursor: &mut impl Buf) -> Result<f64> {
        bytes_fixed::<_>(cursor).map(f64::from_be_bytes)
    }

    fn bytes_fixed<const N: usize>(cursor: &mut impl Buf) -> Result<[u8; N]> {
        if cursor.len() < N {
            Err(InsufficientBytes)
        } else {
            let mut buf = [0u8; N];
            cursor.copy_to_slice(&mut buf);
            Ok(buf)
        }
    }

    pub(super) fn discard(cursor: &mut impl Buf, len: usize) -> Result<()> {
        if cursor.len() < len {
            Err(InsufficientBytes)
        } else {
            cursor.advance(len);
            Ok(())
        }
    }
}
