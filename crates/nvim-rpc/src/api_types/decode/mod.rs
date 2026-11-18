//! Decodes Neovim results into caller-selected Rust representations.
//!
//! [`Decode`] supports borrowed strings and bytes, owned results, and custom
//! types. [`DecodedArray`] converts array elements as they are visited. See
//! those traits and types for examples of each representation.

use nvim_rpc_sys::types as sys;

use mp::decode::{Array, Bin, Buf, Map, Object};

use std::fmt;
use std::marker::PhantomData;
use std::str::Utf8Error;

/// Decodes a value described by marker `T` from backing buffer `B`.
///
/// Choose the Rust result type independently of the API marker:
///
/// ```
/// use nvim_rpc::api_types::decode::Decode;
/// use nvim_rpc::mp::decode::ObjectParser;
/// use nvim_rpc::sys::types;
///
/// let bytes = &b"\xa5hello"[..];
/// let value = || ObjectParser::new(bytes).next_object().unwrap();
/// let borrowed = <&str as Decode<types::String, _>>::decode(value()).unwrap();
/// let owned = <String as Decode<types::String, _>>::decode(value()).unwrap();
/// assert_eq!(borrowed, "hello");
/// assert_eq!(owned, "hello");
/// assert_eq!(borrowed.as_ptr(), bytes[1..].as_ptr());
/// ```
///
/// Borrowed strings and bytes require a slice-backed input and retain its
/// lifetime. `String` and `Vec<u8>` allocate owned storage. Integer conversions
/// check the destination's range, including the full `u64` range for a `u64`
/// output. UTF-8 is checked when producing `&str` or `String`; use bytes to
/// retain strings containing arbitrary byte sequences.
///
/// Applications can delegate decoding and wrap the result:
///
/// ```
/// use nvim_rpc::api_types::decode::{Decode, DecodeError};
/// use nvim_rpc::mp::decode::{Buf, Object};
/// use nvim_rpc::sys::types;
///
/// #[derive(Debug, PartialEq)]
/// struct Count(u64);
/// impl<B: Buf> Decode<types::Object, B> for Count {
///     fn decode(value: Object<B>) -> Result<Self, DecodeError> {
///         <u64 as Decode<types::Object, B>>::decode(value).map(Self)
///     }
/// }
/// assert_eq!(Count::decode(Object::<&[u8]>::U64(u64::MAX)).unwrap(), Count(u64::MAX));
/// ```
///
/// Raw `Object<B>` accepts any marker without validating it. A raw `Array<B>`
/// checks array shape and fixed length where required, but does not decode
/// its elements. [`DecodedArray`] checks elements as they are visited.
/// Signature tuples accept appended fields; fixed arrays and fixed-array
/// tuples require the exact declared length.
pub trait Decode<T, B>: Sized {
    /// Converts one complete MessagePack object into the chosen Rust type.
    fn decode(value: Object<B>) -> Result<Self, DecodeError>;
}

/// A value does not satisfy the requested representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    /// The value has a different shape from the requested representation.
    UnexpectedType(&'static str),
    /// An integer does not fit the requested Rust type.
    IntegerOutOfRange,
    /// An array does not have the required number of elements.
    InvalidLength { expected: usize, actual: usize },
    /// A string cannot be represented as UTF-8 text.
    InvalidUtf8(Utf8Error),
    /// A handle extension has a different type ID.
    InvalidExtension { expected: i8, actual: i8 },
    /// A handle extension payload is truncated or contains trailing data.
    InvalidExtensionData,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedType(expected) => write!(f, "expected {expected}"),
            Self::IntegerOutOfRange => f.write_str("integer is out of range"),
            Self::InvalidLength { expected, actual } => {
                write!(f, "expected {expected} elements, received {actual}")
            }
            Self::InvalidUtf8(e) => e.fmt(f),
            Self::InvalidExtension { expected, actual } => {
                write!(f, "expected extension {expected}, received {actual}")
            }
            Self::InvalidExtensionData => f.write_str("invalid Neovim handle payload"),
        }
    }
}

impl std::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidUtf8(e) => Some(e),
            _ => None,
        }
    }
}

impl<T, B> Decode<T, B> for Object<B> {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        Ok(value)
    }
}

impl<B> Decode<sys::Nil, B> for () {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        match value {
            Object::Nil => Ok(()),
            _ => Err(DecodeError::UnexpectedType("nil")),
        }
    }
}

impl<B> Decode<(), B> for () {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        match value {
            Object::Nil | Object::Array(_) => Ok(()),
            _ => Err(DecodeError::UnexpectedType("nil or argument array")),
        }
    }
}

impl<B> Decode<sys::Bool, B> for bool {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        match value {
            Object::Bool(value) => Ok(value),
            _ => Err(DecodeError::UnexpectedType("boolean")),
        }
    }
}

impl<B> Decode<sys::Int, B> for i8 {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        match value {
            Object::U8(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::U16(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::U32(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::U64(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I8(n) => Ok(n),
            Object::I16(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I32(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I64(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            _ => Err(DecodeError::UnexpectedType("integer")),
        }
    }
}

impl<B> Decode<sys::Object, B> for i8 {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <Self as Decode<sys::Int, B>>::decode(value)
    }
}

impl<B> Decode<sys::Int, B> for i16 {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        match value {
            Object::U8(n) => Ok(Self::from(n)),
            Object::U16(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::U32(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::U64(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I8(n) => Ok(Self::from(n)),
            Object::I16(n) => Ok(n),
            Object::I32(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I64(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            _ => Err(DecodeError::UnexpectedType("integer")),
        }
    }
}

impl<B> Decode<sys::Object, B> for i16 {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <Self as Decode<sys::Int, B>>::decode(value)
    }
}

impl<B> Decode<sys::Int, B> for i32 {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        match value {
            Object::U8(n) => Ok(Self::from(n)),
            Object::U16(n) => Ok(Self::from(n)),
            Object::U32(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::U64(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I8(n) => Ok(Self::from(n)),
            Object::I16(n) => Ok(Self::from(n)),
            Object::I32(n) => Ok(n),
            Object::I64(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            _ => Err(DecodeError::UnexpectedType("integer")),
        }
    }
}

impl<B> Decode<sys::Object, B> for i32 {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <Self as Decode<sys::Int, B>>::decode(value)
    }
}

impl<B> Decode<sys::Int, B> for i64 {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        match value {
            Object::U8(n) => Ok(Self::from(n)),
            Object::U16(n) => Ok(Self::from(n)),
            Object::U32(n) => Ok(Self::from(n)),
            Object::U64(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I8(n) => Ok(Self::from(n)),
            Object::I16(n) => Ok(Self::from(n)),
            Object::I32(n) => Ok(Self::from(n)),
            Object::I64(n) => Ok(n),
            _ => Err(DecodeError::UnexpectedType("integer")),
        }
    }
}

impl<B> Decode<sys::Object, B> for i64 {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <Self as Decode<sys::Int, B>>::decode(value)
    }
}

impl<B> Decode<sys::Int, B> for isize {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        match value {
            Object::U8(n) => Ok(Self::from(n)),
            Object::U16(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::U32(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::U64(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I8(n) => Ok(Self::from(n)),
            Object::I16(n) => Ok(Self::from(n)),
            Object::I32(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I64(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            _ => Err(DecodeError::UnexpectedType("integer")),
        }
    }
}

impl<B> Decode<sys::Object, B> for isize {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <Self as Decode<sys::Int, B>>::decode(value)
    }
}

impl<B> Decode<sys::Int, B> for u8 {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        match value {
            Object::U8(n) => Ok(n),
            Object::U16(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::U32(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::U64(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I8(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I16(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I32(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I64(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            _ => Err(DecodeError::UnexpectedType("integer")),
        }
    }
}

impl<B> Decode<sys::Object, B> for u8 {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <Self as Decode<sys::Int, B>>::decode(value)
    }
}

impl<B> Decode<sys::Int, B> for u16 {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        match value {
            Object::U8(n) => Ok(Self::from(n)),
            Object::U16(n) => Ok(n),
            Object::U32(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::U64(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I8(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I16(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I32(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I64(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            _ => Err(DecodeError::UnexpectedType("integer")),
        }
    }
}

impl<B> Decode<sys::Object, B> for u16 {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <Self as Decode<sys::Int, B>>::decode(value)
    }
}

impl<B> Decode<sys::Int, B> for u32 {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        match value {
            Object::U8(n) => Ok(Self::from(n)),
            Object::U16(n) => Ok(Self::from(n)),
            Object::U32(n) => Ok(n),
            Object::U64(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I8(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I16(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I32(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I64(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            _ => Err(DecodeError::UnexpectedType("integer")),
        }
    }
}

impl<B> Decode<sys::Object, B> for u32 {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <Self as Decode<sys::Int, B>>::decode(value)
    }
}

impl<B> Decode<sys::Int, B> for u64 {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        match value {
            Object::U8(n) => Ok(Self::from(n)),
            Object::U16(n) => Ok(Self::from(n)),
            Object::U32(n) => Ok(Self::from(n)),
            Object::U64(n) => Ok(n),
            Object::I8(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I16(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I32(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I64(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            _ => Err(DecodeError::UnexpectedType("integer")),
        }
    }
}

impl<B> Decode<sys::Object, B> for u64 {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <Self as Decode<sys::Int, B>>::decode(value)
    }
}

impl<B> Decode<sys::Int, B> for usize {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        match value {
            Object::U8(n) => Ok(Self::from(n)),
            Object::U16(n) => Ok(Self::from(n)),
            Object::U32(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::U64(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I8(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I16(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I32(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            Object::I64(n) => Self::try_from(n).map_err(|_| DecodeError::IntegerOutOfRange),
            _ => Err(DecodeError::UnexpectedType("integer")),
        }
    }
}

impl<B> Decode<sys::Object, B> for usize {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <Self as Decode<sys::Int, B>>::decode(value)
    }
}

impl<B> Decode<sys::Float, B> for f64 {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        match value {
            Object::F32(n) => Ok(n.into()),
            Object::F64(n) => Ok(n),
            _ => Err(DecodeError::UnexpectedType("float")),
        }
    }
}

impl<B> Decode<sys::Float, B> for f32 {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <f64 as Decode<sys::Float, B>>::decode(value).map(|n| n as Self)
    }
}

impl<B> Decode<sys::String, B> for Bin<B> {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        match value {
            Object::Str(value) | Object::Bin(value) => Ok(value),
            _ => Err(DecodeError::UnexpectedType("string")),
        }
    }
}

impl<'a> Decode<sys::String, &'a [u8]> for &'a [u8] {
    fn decode(value: Object<&'a [u8]>) -> Result<Self, DecodeError> {
        <Bin<_> as Decode<sys::String, _>>::decode(value).map(Bin::into_inner)
    }
}

impl<'a> Decode<sys::String, &'a [u8]> for &'a str {
    fn decode(value: Object<&'a [u8]>) -> Result<Self, DecodeError> {
        let bytes = <&[u8] as Decode<sys::String, _>>::decode(value)?;
        std::str::from_utf8(bytes).map_err(DecodeError::InvalidUtf8)
    }
}

impl<B: Buf> Decode<sys::String, B> for Vec<u8> {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        let bytes = <Bin<B> as Decode<sys::String, B>>::decode(value)?;
        let mut ret = Vec::with_capacity(bytes.get_ref().len());
        ret.extend(bytes);
        Ok(ret)
    }
}

impl<B: Buf> Decode<sys::String, B> for String {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        let bytes = <Vec<u8> as Decode<sys::String, B>>::decode(value)?;
        String::from_utf8(bytes).map_err(|e| DecodeError::InvalidUtf8(e.utf8_error()))
    }
}

impl<B> Decode<sys::Array, B> for Array<B> {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        match value {
            Object::Array(value) => Ok(value),
            _ => Err(DecodeError::UnexpectedType("array")),
        }
    }
}

impl<M, B> Decode<sys::ArrayOf<M>, B> for Array<B> {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <Self as Decode<sys::Array, B>>::decode(value)
    }
}

impl<M, B: Buf, const N: usize> Decode<sys::FixedArrayOf<M, N>, B> for Array<B> {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        let array = <Self as Decode<sys::Array, B>>::decode(value)?;
        check_len(&array, N)?;
        Ok(array)
    }
}

impl<B> Decode<sys::Dict, B> for Map<B> {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        match value {
            Object::Map(value) => Ok(value),
            _ => Err(DecodeError::UnexpectedType("dictionary")),
        }
    }
}

impl<B> Decode<sys::Object, B> for bool {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <Self as Decode<sys::Bool, B>>::decode(value)
    }
}

impl<B> Decode<sys::Object, B> for f32 {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <Self as Decode<sys::Float, B>>::decode(value)
    }
}

impl<B> Decode<sys::Object, B> for f64 {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <Self as Decode<sys::Float, B>>::decode(value)
    }
}

impl<B: Buf> Decode<sys::Object, B> for String {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <Self as Decode<sys::String, B>>::decode(value)
    }
}

impl<B> Decode<sys::Object, B> for Bin<B> {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <Self as Decode<sys::String, B>>::decode(value)
    }
}

impl<B> Decode<sys::Object, B> for Array<B> {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <Self as Decode<sys::Array, B>>::decode(value)
    }
}

impl<B> Decode<sys::Object, B> for Map<B> {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <Self as Decode<sys::Dict, B>>::decode(value)
    }
}

impl<'a> Decode<sys::Object, &'a [u8]> for &'a str {
    fn decode(value: Object<&'a [u8]>) -> Result<Self, DecodeError> {
        <Self as Decode<sys::String, _>>::decode(value)
    }
}

impl<'a> Decode<sys::Object, &'a [u8]> for &'a [u8] {
    fn decode(value: Object<&'a [u8]>) -> Result<Self, DecodeError> {
        <Self as Decode<sys::String, _>>::decode(value)
    }
}

impl<M, B: Buf, T: Decode<M, B::Slice>> Decode<sys::ArrayOf<M>, B> for Vec<T> {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        DecodedArray::<T, M, B>::decode(value)?.collect()
    }
}

impl<B: Buf, T: Decode<sys::Object, B::Slice>> Decode<sys::Array, B> for Vec<T> {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        let array = <Array<B> as Decode<sys::Array, B>>::decode(value)?;
        array.map(T::decode).collect()
    }
}

impl<B: Buf, T: Decode<sys::Object, B::Slice>> Decode<sys::Object, B> for Vec<T> {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <Self as Decode<sys::Array, B>>::decode(value)
    }
}

fn check_len<B: Buf>(array: &Array<B>, expected: usize) -> Result<(), DecodeError> {
    let actual = array.remaining_len();
    if actual == expected {
        Ok(())
    } else {
        Err(DecodeError::InvalidLength { expected, actual })
    }
}

fn fixed_array<M, B: Buf, T: Decode<M, B::Slice>, const N: usize>(
    array: Array<B>,
) -> Result<[T; N], DecodeError> {
    check_len(&array, N)?;
    let mut ret = [const { None }; N];
    for (dst, value) in ret.iter_mut().zip(array) {
        *dst = Some(T::decode(value)?);
    }
    Ok(ret.map(|value| value.expect("all fixed-array elements were decoded")))
}

impl<M, B: Buf, T: Decode<M, B::Slice>, const N: usize> Decode<sys::FixedArrayOf<M, N>, B>
    for [T; N]
{
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        fixed_array::<M, _, _, N>(<Array<B> as Decode<sys::Array, B>>::decode(value)?)
    }
}

impl<M, B: Buf, T: Decode<M, B::Slice>, const N: usize> Decode<sys::ArrayOf<M>, B> for [T; N] {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        fixed_array::<M, _, _, N>(<Array<B> as Decode<sys::Array, B>>::decode(value)?)
    }
}

/// Decodes array elements one at a time without collecting them.
///
/// Each item is a `Result`, so an invalid element is reported when reached:
///
/// ```
/// use nvim_rpc::api_types::decode::{Decode, DecodeError, DecodedArray};
/// use nvim_rpc::mp::decode::ObjectParser;
/// use nvim_rpc::sys::types;
///
/// let value = ObjectParser::new(&b"\x93\x01\xa1x\x03"[..]).next_object().unwrap();
/// let mut values = DecodedArray::<i64, types::Int, _>::decode(value).unwrap();
/// assert_eq!(values.len(), 3);
/// assert_eq!(values.next().unwrap().unwrap(), 1);
/// assert_eq!(values.next().unwrap(), Err(DecodeError::UnexpectedType("integer")));
/// assert_eq!(values.next().unwrap().unwrap(), 3);
/// assert!(values.next().is_none());
/// ```
///
/// `Vec<T>` provides eager collection through the same [`Decode`] trait.
pub struct DecodedArray<T, M, B> {
    array: Array<B>,
    item: PhantomData<fn() -> (T, M)>,
}

impl<T, M, B> DecodedArray<T, M, B> {
    /// Wraps an array; element validation occurs during iteration.
    pub fn new(array: Array<B>) -> Self {
        Self {
            array,
            item: PhantomData,
        }
    }

    /// Returns the undecoded remainder of the array.
    pub fn into_inner(self) -> Array<B> {
        self.array
    }
}

impl<T, M, B> Decode<sys::ArrayOf<M>, B> for DecodedArray<T, M, B> {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <Array<B> as Decode<sys::Array, B>>::decode(value).map(Self::new)
    }
}

impl<T: Decode<M, B::Slice>, M, B: Buf> Iterator for DecodedArray<T, M, B> {
    type Item = Result<T, DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.array.next().map(T::decode)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.array.size_hint()
    }
}

impl<T: Decode<M, B::Slice>, M, B: Buf> ExactSizeIterator for DecodedArray<T, M, B> {}
impl<T: Decode<M, B::Slice>, M, B: Buf> std::iter::FusedIterator for DecodedArray<T, M, B> {}

macro_rules! _impl_tuple {
    ($n:expr; $($marker:ident $ty:ident),+) => {
        impl<B: Buf, $($marker, $ty: Decode<$marker, B::Slice>),+>
            Decode<($($marker,)+), B> for ($($ty,)+)
        {
            fn decode(value: Object<B>) -> Result<Self, DecodeError> {
                let mut array = <Array<B> as Decode<sys::Array, B>>::decode(value)?;
                if array.remaining_len() < $n {
                    return Err(DecodeError::InvalidLength {
                        expected: $n, actual: array.remaining_len(),
                    });
                }
                Ok(($($ty::decode(array.next().expect("tuple prefix length was checked"))?,)+))
            }
        }

        impl<M, B: Buf, $($ty: Decode<M, B::Slice>),+>
            Decode<sys::FixedArrayOf<M, $n>, B> for ($($ty,)+)
        {
            fn decode(value: Object<B>) -> Result<Self, DecodeError> {
                let mut array = <Array<B> as Decode<sys::Array, B>>::decode(value)?;
                check_len(&array, $n)?;
                Ok(($($ty::decode(array.next().expect("fixed tuple length was checked"))?,)+))
            }
        }
    };
}

_impl_tuple!(1; M0 T0);
_impl_tuple!(2; M0 T0, M1 T1);
_impl_tuple!(3; M0 T0, M1 T1, M2 T2);
_impl_tuple!(4; M0 T0, M1 T1, M2 T2, M3 T3);
_impl_tuple!(5; M0 T0, M1 T1, M2 T2, M3 T3, M4 T4);
_impl_tuple!(6; M0 T0, M1 T1, M2 T2, M3 T3, M4 T4, M5 T5);
_impl_tuple!(7; M0 T0, M1 T1, M2 T2, M3 T3, M4 T4, M5 T5, M6 T6);
_impl_tuple!(8; M0 T0, M1 T1, M2 T2, M3 T3, M4 T4, M5 T5, M6 T6, M7 T7);
_impl_tuple!(9; M0 T0, M1 T1, M2 T2, M3 T3, M4 T4, M5 T5, M6 T6, M7 T7, M8 T8);
_impl_tuple!(10; M0 T0, M1 T1, M2 T2, M3 T3, M4 T4, M5 T5, M6 T6, M7 T7, M8 T8, M9 T9);
_impl_tuple!(11; M0 T0, M1 T1, M2 T2, M3 T3, M4 T4, M5 T5, M6 T6, M7 T7, M8 T8, M9 T9, M10 T10);
_impl_tuple!(12; M0 T0, M1 T1, M2 T2, M3 T3, M4 T4, M5 T5, M6 T6, M7 T7, M8 T8, M9 T9, M10 T10, M11 T11);
_impl_tuple!(13; M0 T0, M1 T1, M2 T2, M3 T3, M4 T4, M5 T5, M6 T6, M7 T7, M8 T8, M9 T9, M10 T10, M11 T11, M12 T12);
_impl_tuple!(14; M0 T0, M1 T1, M2 T2, M3 T3, M4 T4, M5 T5, M6 T6, M7 T7, M8 T8, M9 T9, M10 T10, M11 T11, M12 T12, M13 T13);
_impl_tuple!(15; M0 T0, M1 T1, M2 T2, M3 T3, M4 T4, M5 T5, M6 T6, M7 T7, M8 T8, M9 T9, M10 T10, M11 T11, M12 T12, M13 T13, M14 T14);

#[cfg(test)]
mod tests;
