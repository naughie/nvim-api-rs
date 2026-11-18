use super::{Buf, Seekable};
use super::{InsufficientBytes, read, seek_to_eoo};
use crate::ExtType;

use rmp::Marker;

use std::num::NonZeroUsize;
use std::str::Utf8Error;

/// A decoded value retaining its backing buffer `B`.
///
/// Numeric variants preserve the encoded width. Accessors match their variant
/// exactly; they do not cast between numeric types. Arrays and maps parse their
/// elements lazily.
#[derive(Debug)]
pub enum Object<B> {
    Nil,
    Bool(bool),

    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),

    /// A binary payload retained in the backing buffer.
    Bin(Bin<B>),
    /// A string payload retained without UTF-8 validation.
    ///
    /// Use [`Bin::as_str()`] to validate and borrow it as text.
    Str(Bin<B>),

    Array(Array<B>),
    Map(Map<B>),

    Ext(ExtType, Bin<B>),
}

impl<B> Object<B> {
    /// Returns whether this is a nil value.
    pub fn is_nil(&self) -> bool {
        matches!(self, Self::Nil)
    }

    /// Returns the boolean if this is [`Self::Bool`].
    pub fn to_bool(&self) -> Option<bool> {
        if let &Self::Bool(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns the value if this is [`Self::U8`].
    pub fn to_u8(&self) -> Option<u8> {
        if let &Self::U8(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns the value if this is [`Self::U16`].
    pub fn to_u16(&self) -> Option<u16> {
        if let &Self::U16(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns the value if this is [`Self::U32`].
    pub fn to_u32(&self) -> Option<u32> {
        if let &Self::U32(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns the value if this is [`Self::U64`].
    pub fn to_u64(&self) -> Option<u64> {
        if let &Self::U64(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns the value if this is [`Self::I8`].
    pub fn to_i8(&self) -> Option<i8> {
        if let &Self::I8(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns the value if this is [`Self::I16`].
    pub fn to_i16(&self) -> Option<i16> {
        if let &Self::I16(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns the value if this is [`Self::I32`].
    pub fn to_i32(&self) -> Option<i32> {
        if let &Self::I32(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns the value if this is [`Self::I64`].
    pub fn to_i64(&self) -> Option<i64> {
        if let &Self::I64(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns `Some(n)` if `self` is a `f32` object; otherwise `None`.
    ///
    /// Does not perform any type casting.
    pub fn to_f32(&self) -> Option<f32> {
        if let &Self::F32(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns `Some(n)` if `self` is a `f64` object; otherwise `None`.
    ///
    /// Does not perform any type casting.
    pub fn to_f64(&self) -> Option<f64> {
        if let &Self::F64(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Borrows the payload if this is [`Self::Str`].
    pub fn to_str(&self) -> Option<&Bin<B>> {
        if let Self::Str(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Borrows the payload if this is [`Self::Bin`].
    pub fn to_bin(&self) -> Option<&Bin<B>> {
        if let Self::Bin(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Mutably borrows the element iterator if this is [`Self::Array`].
    pub fn to_array(&mut self) -> Option<&mut Array<B>> {
        if let Self::Array(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Mutably borrows the entry iterator if this is [`Self::Map`].
    pub fn to_map(&mut self) -> Option<&mut Map<B>> {
        if let Self::Map(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns the type ID and borrowed payload if this is [`Self::Ext`].
    pub fn to_ext(&self) -> Option<(ExtType, &Bin<B>)> {
        if let &Self::Ext(ext, ref value) = self {
            Some((ext, value))
        } else {
            None
        }
    }
}

impl<B: Buf> Object<B> {
    // The buffer must contain exactly one complete value.
    fn from_buf_impl(mut buf: B) -> Result<Self, InsufficientBytes> {
        let marker = read::marker(&mut buf)?;

        let obj = match marker {
            Marker::Null | Marker::Reserved => Self::Nil,

            Marker::True => Self::Bool(true),
            Marker::False => Self::Bool(false),

            Marker::FixPos(n) => Self::U8(n),
            Marker::FixNeg(n) => Self::I8(n),

            Marker::U8 => Self::U8(read::u8(&mut buf)?),
            Marker::U16 => Self::U16(read::u16(&mut buf)?),
            Marker::U32 => Self::U32(read::u32(&mut buf)?),
            Marker::U64 => Self::U64(read::u64(&mut buf)?),

            Marker::I8 => Self::I8(read::i8(&mut buf)?),
            Marker::I16 => Self::I16(read::i16(&mut buf)?),
            Marker::I32 => Self::I32(read::i32(&mut buf)?),
            Marker::I64 => Self::I64(read::i64(&mut buf)?),

            Marker::F32 => Self::F32(read::f32(&mut buf)?),
            Marker::F64 => Self::F64(read::f64(&mut buf)?),

            Marker::FixStr(_) => Self::Str(Bin { data: buf }),
            Marker::Str8 => {
                read::discard(&mut buf, 1)?;
                Self::Str(Bin { data: buf })
            }
            Marker::Str16 => {
                read::discard(&mut buf, 2)?;
                Self::Str(Bin { data: buf })
            }
            Marker::Str32 => {
                read::discard(&mut buf, 4)?;
                Self::Str(Bin { data: buf })
            }

            Marker::Bin8 => {
                read::discard(&mut buf, 1)?;
                Self::Bin(Bin { data: buf })
            }
            Marker::Bin16 => {
                read::discard(&mut buf, 2)?;
                Self::Bin(Bin { data: buf })
            }
            Marker::Bin32 => {
                read::discard(&mut buf, 4)?;
                Self::Bin(Bin { data: buf })
            }

            Marker::FixArray(len) => {
                let len = len as usize;
                Self::Array(Array { buf, len })
            }
            Marker::Array16 => {
                let len = read::u16(&mut buf)? as usize;
                Self::Array(Array { buf, len })
            }
            Marker::Array32 => {
                let len = read::u32(&mut buf)? as usize;
                Self::Array(Array { buf, len })
            }

            Marker::FixMap(len) => {
                let len = len as usize;
                Self::Map(Map { buf, len })
            }
            Marker::Map16 => {
                let len = read::u16(&mut buf)? as usize;
                Self::Map(Map { buf, len })
            }
            Marker::Map32 => {
                let len = read::u32(&mut buf)? as usize;
                Self::Map(Map { buf, len })
            }

            Marker::FixExt1
            | Marker::FixExt2
            | Marker::FixExt4
            | Marker::FixExt8
            | Marker::FixExt16 => {
                let ext = ExtType(read::i8(&mut buf)?);
                Self::Ext(ext, Bin { data: buf })
            }
            Marker::Ext8 => {
                read::discard(&mut buf, 1)?;
                let ext = ExtType(read::i8(&mut buf)?);
                Self::Ext(ext, Bin { data: buf })
            }
            Marker::Ext16 => {
                read::discard(&mut buf, 2)?;
                let ext = ExtType(read::i8(&mut buf)?);
                Self::Ext(ext, Bin { data: buf })
            }
            Marker::Ext32 => {
                read::discard(&mut buf, 4)?;
                let ext = ExtType(read::i8(&mut buf)?);
                Self::Ext(ext, Bin { data: buf })
            }
        };
        Ok(obj)
    }

    /// Decodes a buffer without checking its frame boundary.
    ///
    /// `buf` must contain exactly one complete value. Prefer
    /// [`ObjectParser::next_object()`] when the boundary has not been checked.
    ///
    /// # Panics
    ///
    /// An incomplete buffer may panic here or during later array/map iteration.
    pub fn from_buf_unchecked(buf: B) -> Self {
        Self::from_buf_impl(buf).expect("buffer must contain one complete value")
    }
}

/// A string, binary, or extension payload retained in its backing buffer.
#[derive(Debug)]
pub struct Bin<B> {
    data: B,
}

impl<B> Bin<B> {
    /// Borrows the backing buffer.
    pub fn get_ref(&self) -> &B {
        &self.data
    }

    /// Mutably borrows the backing buffer.
    pub fn get_mut(&mut self) -> &mut B {
        &mut self.data
    }

    /// Returns the backing buffer.
    pub fn into_inner(self) -> B {
        self.data
    }
}

impl<B: AsRef<[u8]>> Bin<B> {
    /// Borrows the remaining payload as bytes.
    pub fn as_slice(&self) -> &[u8] {
        self.data.as_ref()
    }

    /// Borrows the remaining payload as text without validating UTF-8.
    ///
    /// # Safety
    ///
    /// The inner byte array must be encoded in UTF-8.
    pub unsafe fn as_str_unchecked(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.data.as_ref()) }
    }

    /// Validates UTF-8 and borrows the remaining payload as text.
    pub fn as_str(&self) -> Result<&str, Utf8Error> {
        std::str::from_utf8(self.data.as_ref())
    }
}

impl<B: Buf> Iterator for Bin<B> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.data.first_byte()
    }
}

/// An iterator over an array's remaining elements.
#[derive(Debug)]
pub struct Array<B> {
    buf: B,
    len: usize,
}

impl<B: AsRef<[u8]>> Array<B> {
    /// Creates an array iterator with a borrowed byte buffer.
    ///
    /// The iterator retains this array's current position and remaining element
    /// count. It decodes elements lazily from the same backing bytes; no elements
    /// are decoded or copied by this method. Its decoded values cannot outlive
    /// this borrow. Advancing the returned iterator does not advance this array.
    pub fn with_borrowed_buffer(&self) -> Array<&[u8]> {
        Array {
            buf: self.buf.as_ref(),
            len: self.len,
        }
    }
}

impl<B: Buf> Array<B> {
    /// Decodes and consumes the next element.
    pub fn next_object(&mut self) -> Option<Object<B::Slice>> {
        if self.len == 0 {
            None
        } else {
            let eoo = seek_to_eoo(&mut self.buf.seekable())
                .expect("array must contain complete elements")
                .get();
            let bytes = self.buf.slice(..eoo);
            self.buf.advance(eoo);
            self.len -= 1;
            Some(Object::from_buf_unchecked(bytes))
        }
    }

    /// Returns the number of remaining elements.
    pub fn remaining_len(&self) -> usize {
        self.len
    }
}

impl<B: Buf> Iterator for Array<B> {
    type Item = Object<B::Slice>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_object()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}
impl<B: Buf> ExactSizeIterator for Array<B> {
    fn len(&self) -> usize {
        self.len
    }
}

/// An iterator over a map's remaining entries.
#[derive(Debug)]
pub struct Map<B> {
    buf: B,
    len: usize,
}

/// Key-value pair of a [`Map`].
#[derive(Debug)]
pub struct MapItem<B> {
    pub key: Object<B>,
    pub value: Object<B>,
}

impl<B: Buf> Map<B> {
    /// Decodes and consumes the next key/value pair.
    pub fn next_pair(&mut self) -> Option<MapItem<B::Slice>> {
        if self.len == 0 {
            None
        } else {
            let eoo = seek_to_eoo(&mut self.buf.seekable())
                .expect("map must contain complete keys")
                .get();
            let key = self.buf.slice(..eoo);
            self.buf.advance(eoo);

            let eoo = seek_to_eoo(&mut self.buf.seekable())
                .expect("map must contain complete values")
                .get();
            let value = self.buf.slice(..eoo);
            self.buf.advance(eoo);

            self.len -= 1;
            Some(MapItem {
                key: Object::from_buf_unchecked(key),
                value: Object::from_buf_unchecked(value),
            })
        }
    }

    /// Returns the number of remaining key/value pairs.
    pub fn remaining_len(&self) -> usize {
        self.len
    }
}

impl<B: Buf> Iterator for Map<B> {
    type Item = MapItem<B::Slice>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_pair()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}
impl<B: Buf> ExactSizeIterator for Map<B> {
    fn len(&self) -> usize {
        self.len
    }
}

/// Decodes complete values from a buffer, advancing after each successful read.
///
/// [`next_object()`](Self::next_object()) returns [`InsufficientBytes`] without
/// advancing when the next value is incomplete. The [`Iterator`] implementation
/// returns `None` in that case, leaving the incomplete bytes in the buffer.
pub struct ObjectParser<B> {
    buf: B,
}

impl<B> ObjectParser<B> {
    /// Creates a parser over the supplied buffer.
    pub fn new(buf: B) -> Self {
        Self { buf }
    }

    /// Borrows the unconsumed buffer.
    pub fn remaining(&self) -> &B {
        &self.buf
    }

    /// Mutably borrows the unconsumed buffer.
    pub fn remaining_mut(&mut self) -> &mut B {
        &mut self.buf
    }

    /// Returns the unconsumed buffer.
    pub fn into_remaining(self) -> B {
        self.buf
    }
}

impl<B: Seekable> ObjectParser<B> {
    /// Returns the next complete value's byte length without advancing.
    pub fn find_eoo(&self) -> Result<NonZeroUsize, InsufficientBytes> {
        seek_to_eoo(&mut self.buf.seekable())
    }
}

impl<B: Buf> ObjectParser<B> {
    /// Decodes and consumes the next complete value.
    pub fn next_object(&mut self) -> Result<Object<B::Slice>, InsufficientBytes> {
        let eoo = self.find_eoo()?.get();
        let bytes = self.buf.slice(..eoo);
        self.buf.advance(eoo);
        Ok(Object::from_buf_unchecked(bytes))
    }
}

impl<B: Buf> Iterator for ObjectParser<B> {
    type Item = Object<B::Slice>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_object().ok()
    }
}
