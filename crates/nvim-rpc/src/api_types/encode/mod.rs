//! Converts Rust arguments to the representation expected by a Neovim API.
//!
//! [`Like`] lets a domain type select an encoding representation without
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
//! # Alternatives
//!
//! [`sys::OneOf<A, B>`] describes a value with either marker `A` or marker `B`.
//! [`Untagged<L, R>`] is its encoding representation: `Left` encodes only its
//! `L` payload, and `Right` encodes only its `R` payload. No variant tag or
//! enclosing array is written. Both payload types must satisfy their respective
//! markers, including the branch that is not selected:
//!
//! ```
//! use nvim_rpc::api_types::encode::{Like, Untagged};
//! use nvim_rpc::sys::types::{self, OneOf};
//! use nvim_rpc::{api, Neovim};
//!
//! type IntOrString = OneOf<types::Int, types::String>;
//!
//! enum Value {
//!     Number(i32),
//!     Text(String),
//! }
//!
//! impl Like<IntOrString> for Value {
//!     type EncodeAs = Untagged<i64, String>;
//!
//!     fn encode(self) -> Self::EncodeAs {
//!         match self {
//!             Self::Number(n) => Untagged::Left(i64::from(n)),
//!             Self::Text(s) => Untagged::Right(s),
//!         }
//!     }
//! }
//!
//! let value = Value::Text("hello".into());
//! let encoded = <Value as Like<IntOrString>>::encode(value);
//! let mut nvim = Neovim::new(Vec::new());
//! nvim.notify::<api::NvimSetVar>(("value", encoded)).unwrap();
//! ```
//!
//! An implementation for `&Value` can instead return `Untagged<i64, &str>` to
//! borrow text. `Untagged` also works with an ordinary marker when both branches
//! satisfy it, such as `Untagged<i32, i64>` for `Int`. An alternative is an
//! [`mp::encode::Encode`] representation; it does not implement
//! [`mp_prim::Primitive`].
//!
//! Nest `OneOf` and `Untagged` for more alternatives. For example,
//! `OneOf<Int, OneOf<String, Array>>` pairs with
//! `Untagged<i64, Untagged<&str, Array<_>>>`. This composition supports three,
//! four, five, six, or more alternatives without a separate enum for each arity.
//! Order and nesting are part of the Rust type; the library does not normalize
//! these expressions or add wire tags for nested branches.
//!
//! A custom `Like<OneOf<...>>` implementation does not implicitly implement
//! `Like<Object>`. Select its conversion explicitly, as above, when passing it
//! to an `Object` argument. The resulting `Untagged` representation is itself
//! object-compatible when all of its branches are. Typed collections can use
//! `ArrayOf<OneOf<...>>` to select that conversion for every element.
//!
//! A representation cannot claim a union whose branch markers it violates:
//!
//! ```compile_fail
//! use nvim_rpc::api_types::encode::{Like, Untagged};
//! use nvim_rpc::sys::types::{self, OneOf};
//!
//! let value: Untagged<i64, bool> = Untagged::Left(7);
//! let _ = <_ as Like<OneOf<types::Int, types::String>>>::encode(value);
//! ```
//!
//! # Collections
//!
//! Collections can be supplied as exact-size iterators. For example,
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
//! Borrowed slices and array references apply [`Like`] to each element for
//! [`sys::ArrayOf`] arguments. Array references also support [`sys::FixedArrayOf`]
//! when their length matches the marker. The conversion preserves each borrow:
//!
//! ```
//! use nvim_rpc::api_types::encode::Like;
//! use nvim_rpc::{api, Buffer, Neovim, sys};
//!
//! struct Line(String);
//!
//! impl<'a> Like<sys::types::String> for &'a Line {
//!     type EncodeAs = &'a str;
//!     fn encode(self) -> Self::EncodeAs { &self.0 }
//! }
//!
//! let lines = [Line("first".into()), Line("second".into())];
//! let mut nvim = Neovim::new(Vec::new());
//! nvim.notify::<api::NvimBufSetLines>((Buffer::CURRENT, 0i64, -1i64, true, &lines[..]))
//!     .unwrap();
//! ```
//!
//! Use [`Convert`] around `Array`, `ArrayIter`, tuples, `Map`, or `MapIter` to
//! compose element conversions for those representations. Collections of
//! compatible representations, including `Untagged`, encode directly.
//! Iterated elements convert lazily without collecting; tuples convert their
//! elements when [`Like::encode()`] is called.
//! For an untyped [`sys::Array`] or [`sys::Object`], array elements must implement
//! `Like<Object>`. A custom type implementing only `Like<String>` can be used
//! with `ArrayOf<String>`. An untyped array requires either its own
//! `Like<Object>` implementation or an explicit conversion to a representation
//! accepted as an `Object`.
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
//!
//! `Convert` dictionaries apply `Like<String>` to keys and `Like<Object>` to
//! values. The key requirement also applies after conversion:
//!
//! ```compile_fail
//! use nvim_rpc::api_types::encode::Convert;
//! use nvim_rpc::mp::encode::primitive::Map;
//! use nvim_rpc::{api, Neovim};
//!
//! let mut nvim = Neovim::new(Vec::new());
//! nvim.request::<api::NvimGetOptionValue>(("number", Convert(Map([(1i64, true)]))))
//!     .unwrap();
//! ```
//!
//! A `Convert` map used as an `Object` applies `Like<Object>` to both keys and
//! values, so its keys need not be strings.

use super::BaseType;
use super::sys;
use mp::encode::{Encode, primitive as mp_prim};

mod untagged;
pub use mp::encode::Untagged;

mod array;
mod boolean;
mod dict;
mod float;
mod int;
mod nil;
mod string;

/// Applies [`Like`] to the elements of an array, tuple, or map.
///
/// Wrap [`mp_prim::Array`], [`mp_prim::ArrayIter`], a tuple through arity 15
/// (including `()`), [`mp_prim::Map`], or [`mp_prim::MapIter`]. Arrays use their
/// declared element marker; untyped arrays and object maps use `Object`.
/// Dictionaries convert keys as `String` and values as `Object`. Nested
/// collections can use nested adapters.
///
/// The adapter does not collect or allocate; iterator elements convert lazily.
/// Tuple elements are converted when [`Like::encode`] is called. Values are
/// borrowed or moved according to the wrapped collection's iterator.
///
/// For example, a dictionary can contain a handle without manually extracting
/// its integer ID:
///
/// ```
/// use nvim_rpc::api_types::encode::Convert;
/// use nvim_rpc::mp::encode::primitive::Map;
/// use nvim_rpc::{api, Buffer, Neovim};
///
/// let mut nvim = Neovim::new(Vec::new());
/// let opts = Convert(Map([("buf", Buffer::new(7))]));
/// nvim.request::<api::NvimGetOptionValue>(("modified", opts)).unwrap();
/// ```
///
/// [`sys::FixedArrayOf`] accepts the existing array and tuple shapes:
/// `Array([T; N])`, `Array(&[T; N])`, `ArrayIter([T; N].into_iter())`, or a
/// tuple of the required arity. For an array iterator, `N` is the original
/// array's length; pass it unconsumed to encode all `N` elements. A different
/// array length or tuple arity is rejected:
///
/// ```compile_fail
/// use nvim_rpc::api_types::encode::Convert;
/// use nvim_rpc::mp::encode::primitive::Array;
/// use nvim_rpc::{api, Neovim, Window};
///
/// let mut nvim = Neovim::new(Vec::new());
/// nvim.notify::<api::NvimWinSetCursor>((Window::CURRENT, Convert(Array([1i64]))))
///     .unwrap();
/// ```
///
/// An arbitrary exact-size iterator cannot supply a fixed-length argument,
/// even when its runtime length matches:
///
/// ```compile_fail
/// use nvim_rpc::api_types::encode::Convert;
/// use nvim_rpc::mp::encode::primitive::ArrayIter;
/// use nvim_rpc::{api, Neovim, Window};
///
/// let mut nvim = Neovim::new(Vec::new());
/// nvim.notify::<api::NvimWinSetCursor>((Window::CURRENT, Convert(ArrayIter(1u8..3))))
///     .unwrap();
/// ```
pub struct Convert<T>(pub T);

/// An encoding representation accepted for Neovim's type marker `T`.
///
/// For example, `&str` implements `LikeBaseType<sys::types::String>`, and `i64`
/// implements `LikeBaseType<sys::types::Int>`. Representations may also compose
/// compatible alternatives with [`Untagged`], recursively. They receive an
/// identity [`Like`] implementation.
pub trait LikeBaseType<T: BaseType>: Encode {}

/// Converts an application value into an encoding compatible with `T`.
///
/// The associated representation controls ownership: returning `&'a str`
/// preserves a borrow; returning `String` transfers an owned value.
pub trait Like<T: BaseType> {
    /// The representation produced by [`encode()`](Self::encode()).
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
