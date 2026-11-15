//! Buffer, window, and tabpage IDs with their generated API methods.
//!
//! A handle is a value; `sys::ext_types::Buffer` is the marker used in a
//! function signature. Pass the handle as the first argument to its descriptor:
//!
//! ```
//! use nvim_rpc::{api, Buffer, Neovim};
//! let mut nvim = Neovim::new(Vec::new());
//! let buffer = Buffer::new(7);
//! let pending = nvim.request::<api::NvimBufGetLines>((buffer, 0i64, -1i64, true)).unwrap();
//! ```
//!
//! Handle methods construct a [`crate::Call`] for manual envelope encoding:
//!
//! ```
//! use nvim_rpc::Buffer;
//!
//! let buffer = Buffer::new(7);
//! let call = buffer.get_lines(0i64, -1i64, true);
//! assert_eq!(call.into_params(), (buffer, 0, -1, true));
//! assert_eq!(Buffer::CURRENT.id(), 0);
//! ```
//!
//! Use [`crate::Call::into_request()`] or [`crate::Call::into_notification()`] when
//! assigning IDs and writing envelopes yourself. Ordinary [`crate::Neovim`]
//! requests use the descriptor and tuple form above.
//!
//! Handles convert to integer primitives at encoding time. Decoding accepts an
//! integer or an extension with the matching handle tag and one integer payload.
//! Constructing a handle does not query whether that ID is still valid in Neovim.

use crate::api_types::encode::{Like, LikeBaseType};
use crate::api_types::{Decode, DecodeError};
use crate::sys;

use mp::decode::{Buf, Object, ObjectParser};

use std::fmt;

/// A Neovim `Buffer` handle, stored as an integer ID.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Buffer(pub i64);

impl Buffer {
    /// ID zero, which Neovim interprets as the current handle.
    pub const CURRENT: Self = Self(0);

    /// Wraps an ID without checking it against the server.
    pub const fn new(id: i64) -> Self {
        Self(id)
    }

    /// The integer ID sent to Neovim.
    pub const fn id(self) -> i64 {
        self.0
    }
}

impl From<i64> for Buffer {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<Buffer> for i64 {
    fn from(value: Buffer) -> Self {
        value.0
    }
}

impl fmt::Display for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Like<sys::ext_types::Buffer> for Buffer {
    type EncodeAs = i64;

    fn encode(self) -> Self::EncodeAs {
        self.0
    }
}

impl Like<sys::ext_types::Buffer> for &Buffer {
    type EncodeAs = i64;

    fn encode(self) -> Self::EncodeAs {
        self.0
    }
}

impl Like<sys::types::Object> for Buffer {
    type EncodeAs = i64;

    fn encode(self) -> Self::EncodeAs {
        self.0
    }
}

impl Like<sys::types::Object> for &Buffer {
    type EncodeAs = i64;

    fn encode(self) -> Self::EncodeAs {
        self.0
    }
}

impl LikeBaseType<sys::ext_types::Buffer> for i64 {}
impl LikeBaseType<sys::ext_types::Buffer> for &i64 {}
impl LikeBaseType<sys::ext_types::Buffer> for i32 {}
impl LikeBaseType<sys::ext_types::Buffer> for &i32 {}

impl<B: Buf> Decode<sys::ext_types::Buffer, B> for Buffer {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        let id = if let Object::Ext(tag, bytes) = value {
            let expected = sys::ext_types::Buffer::ID as i8;
            if tag.0 != expected {
                return Err(DecodeError::InvalidExtension {
                    expected,
                    actual: tag.0,
                });
            }
            let mut parser = ObjectParser::new(bytes.into_inner());
            let value = parser
                .next_object()
                .map_err(|_| DecodeError::InvalidExtensionData)?;
            if parser.remaining().len() != 0 {
                return Err(DecodeError::InvalidExtensionData);
            }
            <i64 as Decode<sys::types::Int, B::Slice>>::decode(value)?
        } else {
            <i64 as Decode<sys::types::Int, B>>::decode(value)?
        };
        Ok(Self(id))
    }
}

impl<B: Buf> Decode<sys::types::Object, B> for Buffer {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <Self as Decode<sys::ext_types::Buffer, B>>::decode(value)
    }
}

/// A Neovim `Window` handle, stored as an integer ID.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Window(pub i64);

impl Window {
    /// ID zero, which Neovim interprets as the current handle.
    pub const CURRENT: Self = Self(0);

    /// Wraps an ID without checking it against the server.
    pub const fn new(id: i64) -> Self {
        Self(id)
    }

    /// The integer ID sent to Neovim.
    pub const fn id(self) -> i64 {
        self.0
    }
}

impl From<i64> for Window {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<Window> for i64 {
    fn from(value: Window) -> Self {
        value.0
    }
}

impl fmt::Display for Window {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Like<sys::ext_types::Window> for Window {
    type EncodeAs = i64;

    fn encode(self) -> Self::EncodeAs {
        self.0
    }
}

impl Like<sys::ext_types::Window> for &Window {
    type EncodeAs = i64;

    fn encode(self) -> Self::EncodeAs {
        self.0
    }
}

impl Like<sys::types::Object> for Window {
    type EncodeAs = i64;

    fn encode(self) -> Self::EncodeAs {
        self.0
    }
}

impl Like<sys::types::Object> for &Window {
    type EncodeAs = i64;

    fn encode(self) -> Self::EncodeAs {
        self.0
    }
}

impl LikeBaseType<sys::ext_types::Window> for i64 {}
impl LikeBaseType<sys::ext_types::Window> for &i64 {}
impl LikeBaseType<sys::ext_types::Window> for i32 {}
impl LikeBaseType<sys::ext_types::Window> for &i32 {}

impl<B: Buf> Decode<sys::ext_types::Window, B> for Window {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        let id = if let Object::Ext(tag, bytes) = value {
            let expected = sys::ext_types::Window::ID as i8;
            if tag.0 != expected {
                return Err(DecodeError::InvalidExtension {
                    expected,
                    actual: tag.0,
                });
            }
            let mut parser = ObjectParser::new(bytes.into_inner());
            let value = parser
                .next_object()
                .map_err(|_| DecodeError::InvalidExtensionData)?;
            if parser.remaining().len() != 0 {
                return Err(DecodeError::InvalidExtensionData);
            }
            <i64 as Decode<sys::types::Int, B::Slice>>::decode(value)?
        } else {
            <i64 as Decode<sys::types::Int, B>>::decode(value)?
        };
        Ok(Self(id))
    }
}

impl<B: Buf> Decode<sys::types::Object, B> for Window {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <Self as Decode<sys::ext_types::Window, B>>::decode(value)
    }
}

/// A Neovim `Tabpage` handle, stored as an integer ID.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tabpage(pub i64);

impl Tabpage {
    /// ID zero, which Neovim interprets as the current handle.
    pub const CURRENT: Self = Self(0);

    /// Wraps an ID without checking it against the server.
    pub const fn new(id: i64) -> Self {
        Self(id)
    }

    /// The integer ID sent to Neovim.
    pub const fn id(self) -> i64 {
        self.0
    }
}

impl From<i64> for Tabpage {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<Tabpage> for i64 {
    fn from(value: Tabpage) -> Self {
        value.0
    }
}

impl fmt::Display for Tabpage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Like<sys::ext_types::Tabpage> for Tabpage {
    type EncodeAs = i64;

    fn encode(self) -> Self::EncodeAs {
        self.0
    }
}

impl Like<sys::ext_types::Tabpage> for &Tabpage {
    type EncodeAs = i64;

    fn encode(self) -> Self::EncodeAs {
        self.0
    }
}

impl Like<sys::types::Object> for Tabpage {
    type EncodeAs = i64;

    fn encode(self) -> Self::EncodeAs {
        self.0
    }
}

impl Like<sys::types::Object> for &Tabpage {
    type EncodeAs = i64;

    fn encode(self) -> Self::EncodeAs {
        self.0
    }
}

impl LikeBaseType<sys::ext_types::Tabpage> for i64 {}
impl LikeBaseType<sys::ext_types::Tabpage> for &i64 {}
impl LikeBaseType<sys::ext_types::Tabpage> for i32 {}
impl LikeBaseType<sys::ext_types::Tabpage> for &i32 {}

impl<B: Buf> Decode<sys::ext_types::Tabpage, B> for Tabpage {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        let id = if let Object::Ext(tag, bytes) = value {
            let expected = sys::ext_types::Tabpage::ID as i8;
            if tag.0 != expected {
                return Err(DecodeError::InvalidExtension {
                    expected,
                    actual: tag.0,
                });
            }
            let mut parser = ObjectParser::new(bytes.into_inner());
            let value = parser
                .next_object()
                .map_err(|_| DecodeError::InvalidExtensionData)?;
            if parser.remaining().len() != 0 {
                return Err(DecodeError::InvalidExtensionData);
            }
            <i64 as Decode<sys::types::Int, B::Slice>>::decode(value)?
        } else {
            <i64 as Decode<sys::types::Int, B>>::decode(value)?
        };
        Ok(Self(id))
    }
}

impl<B: Buf> Decode<sys::types::Object, B> for Tabpage {
    fn decode(value: Object<B>) -> Result<Self, DecodeError> {
        <Self as Decode<sys::ext_types::Tabpage, B>>::decode(value)
    }
}
