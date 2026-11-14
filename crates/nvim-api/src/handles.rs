//! Buffer, window, and tabpage IDs with their generated API methods.
//!
//! A handle is a value; `sys::ext_types::Buffer` is the marker used in a
//! function signature. Pass the handle as the first argument to its descriptor:
//!
//! ```
//! use nvim_api::{api, Buffer, Neovim};
//! let mut nvim = Neovim::new(Vec::new());
//! let buffer = Buffer::new(7);
//! let pending = nvim.request::<api::NvimBufGetLines>((buffer, 0i64, -1i64, true)).unwrap();
//! ```
//!
//! Handle methods construct a [`crate::Call`] for manual envelope encoding:
//!
//! ```
//! use nvim_api::Buffer;
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

macro_rules! _handle {
    ($name:ident) => {
        #[doc = concat!("A Neovim `", stringify!($name), "` handle, stored as an integer ID.")]
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub i64);

        impl $name {
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

        impl From<i64> for $name {
            fn from(value: i64) -> Self {
                Self(value)
            }
        }

        impl From<$name> for i64 {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl Like<sys::ext_types::$name> for $name {
            type EncodeAs = i64;

            fn encode(self) -> Self::EncodeAs {
                self.0
            }
        }

        impl Like<sys::ext_types::$name> for &$name {
            type EncodeAs = i64;

            fn encode(self) -> Self::EncodeAs {
                self.0
            }
        }

        impl Like<sys::types::Object> for $name {
            type EncodeAs = i64;

            fn encode(self) -> Self::EncodeAs {
                self.0
            }
        }

        impl Like<sys::types::Object> for &$name {
            type EncodeAs = i64;

            fn encode(self) -> Self::EncodeAs {
                self.0
            }
        }

        impl LikeBaseType<sys::ext_types::$name> for i64 {}
        impl LikeBaseType<sys::ext_types::$name> for &i64 {}
        impl LikeBaseType<sys::ext_types::$name> for i32 {}
        impl LikeBaseType<sys::ext_types::$name> for &i32 {}

        impl<B: Buf> Decode<sys::ext_types::$name, B> for $name {
            fn decode(value: Object<B>) -> Result<Self, DecodeError> {
                let id = if let Object::Ext(tag, bytes) = value {
                    let expected = sys::ext_types::$name::ID as i8;
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

        impl<B: Buf> Decode<sys::types::Object, B> for $name {
            fn decode(value: Object<B>) -> Result<Self, DecodeError> {
                <Self as Decode<sys::ext_types::$name, B>>::decode(value)
            }
        }
    };
}

_handle!(Buffer);
_handle!(Window);
_handle!(Tabpage);
