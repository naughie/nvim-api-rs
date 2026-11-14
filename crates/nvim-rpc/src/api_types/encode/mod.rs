//! Converts Rust arguments to the representation expected by a Neovim API.
//!
//! [`Like`] lets a domain type select a primitive representation without
//! becoming a MessagePack primitive itself:
//!
//! ```
//! use nvim_rpc::api_types::encode::Like;
//! use nvim_rpc::{api, Neovim, sys};
//!
//! struct Line<'a>(&'a str);
//!
//! impl<'a> Like<sys::types::String> for Line<'a> {
//!     type EncodeAs = &'a str;
//!     fn encode(self) -> Self::EncodeAs { self.0 }
//! }
//!
//! let mut nvim = Neovim::new(Vec::new());
//! nvim.notify::<api::NvimSetCurrentLine>((Line("borrowed"),)).unwrap();
//! assert!(!nvim.into_inner().is_empty());
//! ```
//!
//! Collections can also be supplied as exact-size iterators. For example,
//! borrowing the keys of a map does not require collecting another array:
//!
//! ```
//! use nvim_rpc::mp::encode::primitive::ArrayIter;
//! use nvim_rpc::{api, Buffer, Neovim};
//! use std::collections::BTreeMap;
//!
//! let entries = BTreeMap::from([("alpha".to_owned(), ()), ("beta".to_owned(), ())]);
//! let lines = ArrayIter(entries.keys().map(String::as_str));
//! let mut nvim = Neovim::new(Vec::new());
//! nvim.notify::<api::NvimBufSetLines>((Buffer::CURRENT, 0i64, -1i64, true, lines)).unwrap();
//! ```
//!
//! `Array` and `ArrayIter` require already-compatible primitive elements.
//! A collection of custom `Like` values must map those values to their
//! primitive representations before wrapping the iterator.
//!
//! Use [`mp::encode::primitive::Map`] or [`mp::encode::primitive::MapIter`] for
//! dictionaries, including empty options:
//!
//! ```
//! use nvim_rpc::mp::encode::primitive::Map;
//! use nvim_rpc::{api, Neovim};
//!
//! let mut nvim = Neovim::new(Vec::new());
//! let opts = Map::<[(&str, &str); 0]>([]);
//! nvim.request::<api::NvimGetOptionValue>(("number", opts)).unwrap();
//! ```
//!
//! `()` represents an empty argument list or array, so it cannot supply a
//! dictionary argument:
//!
//! ```compile_fail
//! use nvim_rpc::{api, Neovim};
//!
//! let mut nvim = Neovim::new(Vec::new());
//! nvim.request::<api::NvimGetOptionValue>(("number", ())).unwrap();
//! ```
//!
//! Dictionary keys must be compatible with [`crate::sys::types::String`]:
//!
//! ```compile_fail
//! use nvim_rpc::mp::encode::primitive::Map;
//! use nvim_rpc::{api, Neovim};
//!
//! let mut nvim = Neovim::new(Vec::new());
//! nvim.request::<api::NvimGetOptionValue>(("number", Map([(1i64, true)]))).unwrap();
//! ```

use super::BaseType;
use super::sys;
use mp::encode::{Primitive, primitive as mp_prim};

mod array;
mod boolean;
mod dict;
mod float;
mod int;
mod nil;
mod string;

/// A primitive representation accepted for Neovim's type marker `T`.
///
/// For example, `&str` implements `LikeBaseType<sys::types::String>`, and `i64`
/// implements `LikeBaseType<sys::types::Int>`. Compatible primitives receive an
/// identity [`Like`] implementation.
pub trait LikeBaseType<T: BaseType>: Primitive {}

/// Converts an application value into a primitive compatible with `T`.
///
/// The associated representation controls ownership: returning `&'a str`
/// preserves a borrow; returning `String` transfers an owned value.
pub trait Like<T: BaseType> {
    /// The primitive produced by [`encode()`](Self::encode()).
    type EncodeAs: LikeBaseType<T>;
    /// Consumes the argument and produces its encoding representation.
    fn encode(self) -> Self::EncodeAs;
}

impl<T: BaseType, P: LikeBaseType<T>> Like<T> for P {
    type EncodeAs = P;
    fn encode(self) -> Self::EncodeAs {
        self
    }
}
