//! Built-in MessagePack representations.
//!
//! Integers, floats, booleans, strings, byte slices, and tuples encode directly.
//! Tuples and `()` encode as arrays; [`Nil`] encodes MessagePack nil. Use [`Array`]
//! or [`ArrayIter`] for homogeneous sequences and [`Map`] or [`MapIter`] for
//! key/value pairs. Their iterators must have an exact length for the wire header.
//!
//! ```
//! use rmp_bytes::encode::WriterSyncExt as _;
//! use rmp_bytes::encode::primitive::{ArrayIter, Nil};
//!
//! let mut bytes = Vec::new();
//! bytes.write_primitive((Nil, ArrayIter((1u8..=3).map(|n| n * 2)))).consume();
//! assert_eq!(bytes, b"\x92\xc0\x93\x02\x04\x06");
//! ```
//!
//! [`Primitive`] is sealed: downstream types should convert to these
//! representations. There is no blanket `Option<T>` encoding; choose explicitly
//! whether absence means nil, an omitted field, or another application value.
//! Values exceeding MessagePack's 32-bit length limits are truncated by the
//! built-in collection encoders; strings are truncated at a UTF-8 boundary.

mod nil;
pub use nil::Nil;

mod bool;

mod tuple;

mod func;
pub use func::Func;

mod float;
mod i16;
mod i32;
mod i64;
mod i8;
mod isize;
mod u16;
mod u32;
mod u64;
mod u8;
mod usize;

mod str;

mod bin;
pub use bin::BinIter;

mod array;
pub use array::{Array, ArrayIter};

mod map;
pub use map::{Map, MapIter};

mod infallible;

const MAX_LEN: usize = 0xffff_ffff;
const MAX_LEN_U32: u32 = 0xffff_ffff;

/// The remaining chunks of one encoded value.
///
/// A chunk can own its bytes or borrow the input. Keep it alive until the writer
/// has finished using [`AsRef::as_ref()`].
pub trait State {
    /// A byte chunk yielded by this state.
    type Item: AsRef<[u8]>;

    /// Advances to the next chunk, or returns `None` when encoding is complete.
    fn next(&mut self) -> Option<Self::Item>;
}

/// A built-in value that can create a MessagePack encoding state.
///
/// This trait is sealed. For a domain type, pass its primitive representation
/// to a writer, such as a handle's integer ID or a record's tuple of fields.
///
/// ```
/// use rmp_bytes::encode::primitive::{Primitive, State as _};
///
/// let mut state = "hi".into_state();
/// assert_eq!(state.next().unwrap().as_ref(), b"\xa2");
/// assert_eq!(state.next().unwrap().as_ref(), b"hi");
/// assert!(state.next().is_none());
/// ```
pub trait Primitive: Sealed {
    /// State that produces this value's byte chunks.
    type State: State;

    /// Consumes the value and starts encoding it.
    fn into_state(self) -> Self::State;
}

/// A primitive whose outer MessagePack value is an array.
pub trait ArrayLike: Primitive {}
/// A primitive whose outer MessagePack value is a string.
pub trait StrLike: Primitive {}

pub(crate) use _priv::Sealed;
mod _priv {
    pub trait Sealed {}
}

mod data {
    use super::num::Num;
    use rmp::Marker;

    pub(super) fn marker(marker: Marker) -> [u8; 1] {
        [marker.to_u8()]
    }

    pub(super) fn len<T: Num, const N: usize>(marker: Marker, len: T) -> [u8; N] {
        let num_bytes = len.to_be_bytes();
        let num_bytes = num_bytes.as_ref();
        debug_assert_eq!(num_bytes.len() + 1, N);

        let mut bytes = [0u8; N];

        bytes[0] = marker.to_u8();
        bytes[1..].copy_from_slice(num_bytes);

        bytes
    }
}

mod num {
    use rmp::Marker;

    pub(super) fn fix_pos(n: u8) -> [u8; 1] {
        [Marker::FixPos(n).to_u8()]
    }

    pub(super) fn fix_neg(n: i8) -> [u8; 1] {
        [Marker::FixNeg(n).to_u8()]
    }

    pub(super) fn with_marker<T: Num, const N: usize>(n: T) -> [u8; N] {
        let num_bytes = n.to_be_bytes();
        let num_bytes = num_bytes.as_ref();
        debug_assert_eq!(num_bytes.len() + 1, N);

        let mut bytes = [0u8; N];

        bytes[0] = T::marker().to_u8();
        bytes[1..].copy_from_slice(num_bytes);

        bytes
    }

    pub(super) trait Num: Copy {
        fn marker() -> Marker;
        fn to_be_bytes(self) -> impl AsRef<[u8]>;
    }

    impl Num for u8 {
        fn marker() -> Marker {
            Marker::U8
        }
        fn to_be_bytes(self) -> impl AsRef<[u8]> {
            [self]
        }
    }

    impl Num for u16 {
        fn marker() -> Marker {
            Marker::U16
        }
        fn to_be_bytes(self) -> impl AsRef<[u8]> {
            self.to_be_bytes()
        }
    }

    impl Num for u32 {
        fn marker() -> Marker {
            Marker::U32
        }
        fn to_be_bytes(self) -> impl AsRef<[u8]> {
            self.to_be_bytes()
        }
    }

    impl Num for u64 {
        fn marker() -> Marker {
            Marker::U64
        }
        fn to_be_bytes(self) -> impl AsRef<[u8]> {
            self.to_be_bytes()
        }
    }

    impl Num for i8 {
        fn marker() -> Marker {
            Marker::I8
        }
        fn to_be_bytes(self) -> impl AsRef<[u8]> {
            [self as u8]
        }
    }

    impl Num for i16 {
        fn marker() -> Marker {
            Marker::I16
        }
        fn to_be_bytes(self) -> impl AsRef<[u8]> {
            self.to_be_bytes()
        }
    }

    impl Num for i32 {
        fn marker() -> Marker {
            Marker::I32
        }
        fn to_be_bytes(self) -> impl AsRef<[u8]> {
            self.to_be_bytes()
        }
    }

    impl Num for i64 {
        fn marker() -> Marker {
            Marker::I64
        }
        fn to_be_bytes(self) -> impl AsRef<[u8]> {
            self.to_be_bytes()
        }
    }

    impl Num for f32 {
        fn marker() -> Marker {
            Marker::F32
        }
        fn to_be_bytes(self) -> impl AsRef<[u8]> {
            self.to_be_bytes()
        }
    }

    impl Num for f64 {
        fn marker() -> Marker {
            Marker::F64
        }
        fn to_be_bytes(self) -> impl AsRef<[u8]> {
            self.to_be_bytes()
        }
    }
}

#[cfg(test)]
fn encode_to_vec<P: Primitive>(v: P) -> Vec<u8> {
    use super::WriterSyncExt as _;
    let mut buf: Vec<u8> = Vec::new();
    let chunks = buf.write_primitive(v);

    chunks.consume();

    buf
}

#[cfg(test)]
fn count_encoded_with_marker<P: Primitive>(v: P, marker: &[u8]) -> usize {
    use super::WriterSync;
    use super::WriterSyncExt as _;

    struct Counter<'a> {
        count: usize,
        marker: &'a [u8],
        buf: Vec<u8>,
    }

    impl WriterSync for Counter<'_> {
        type Result = ();
        fn write_all(&mut self, buf: &[u8]) -> Self::Result {
            self.count = self.count.strict_add(buf.len());

            if self.buf.len() < self.marker.len() {
                if self.buf.len() + buf.len() < self.marker.len() {
                    self.buf.extend_from_slice(buf);
                } else {
                    let len = self.marker.len() - self.buf.len();
                    self.buf.extend_from_slice(&buf[..len]);
                }
            }
        }

        fn flush(&mut self) {}
    }

    let mut counter = Counter {
        count: 0,
        marker,
        buf: vec![],
    };
    let chunks = counter.write_primitive(v);
    chunks.consume();
    assert_eq!(counter.buf, counter.marker);

    counter.count
}
