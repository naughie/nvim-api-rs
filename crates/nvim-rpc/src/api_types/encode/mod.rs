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
//! `Untagged<i64, Untagged<&str, Array<_>>>`.
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
//! Use [`Map`] to transform an iterable's elements before their `Like`
//! conversions, or [`MapRef`] to transform borrowed elements and retain the
//! source. For example, `Map(ids, Buffer::new)` converts IDs into handles;
//! `MapRef(&lines, String::as_str)` borrows strings. Both preserve the source's
//! exact length and apply the function lazily, without an intermediate collection.
//!
//! Use [`Convert`] around `Array`, `ArrayIter`, tuples, [`mp_prim::Map`], or `MapIter` to
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

/// Maps an iterable's elements before converting them to the requested API type.
///
/// Consumes an exactly sized iterable `A` and calls `F` once per visited element
/// during encoding, without an intermediate collection.
///
/// ```
/// use nvim_rpc::api_types::encode::Map;
/// use nvim_rpc::{api, Buffer, Neovim};
///
/// let mut nvim = Neovim::new(Vec::new());
/// let buffers = Map([7, 9], Buffer::new);
/// nvim.notify::<api::NvimSetVar>(("buffers", buffers)).unwrap();
/// ```
///
/// Supports `ArrayOf<B>`, `Array`, and `Object`; the mapped values must implement
/// `Like<B>` or, for untyped arrays, `Like<Object>`. Sources `[T; N]`, `&[T; N]`,
/// `Array<[T; N]>`, and `Array<&[T; N]>` also preserve their length for
/// `FixedArrayOf<B, N>`. A different length is rejected:
///
/// ```compile_fail
/// use nvim_rpc::api_types::encode::Map;
/// use nvim_rpc::{api, Neovim, Window};
///
/// let mut nvim = Neovim::new(Vec::new());
/// nvim.notify::<api::NvimWinSetCursor>((Window::CURRENT, Map([1i64], |n| n + 1)))
///     .unwrap();
/// ```
///
/// Other iterable sources support dynamically sized arrays, even when their
/// runtime length happens to match a fixed-length marker. This adapter is
/// distinct from the MessagePack dictionary representation
/// [`mp::encode::primitive::Map`].
pub struct Map<A, F>(pub A, pub F);

/// Maps shared references to an iterable's elements without consuming the source.
///
/// `F` receives `&'s T` and may return a value borrowing that element. Conversion
/// remains lazy and requires an exactly sized iterator, as with [`Map`].
/// Borrowed `[T; N]` and `Array<[T; N]>` sources also support
/// `FixedArrayOf<B, N>`.
///
/// ```
/// use nvim_rpc::api_types::encode::MapRef;
/// use nvim_rpc::{api, Buffer, Neovim};
///
/// let lines = [String::from("first"), String::from("second")];
/// let mut nvim = Neovim::new(Vec::new());
/// nvim.notify::<api::NvimBufSetLines>((
///     Buffer::CURRENT, 0i64, -1i64, true, MapRef(&lines, String::as_str),
/// )).unwrap();
/// assert_eq!(lines[0], "first");
/// ```
pub struct MapRef<'s, A: ?Sized, F>(pub &'s A, pub F);

/// Applies [`Like`] to the elements of an array, tuple, or map.
///
/// Wrap [`mp_prim::Array`], [`mp_prim::ArrayIter`], a tuple through arity 15
/// (including `()`), [`mp_prim::Map`], or [`mp_prim::MapIter`]. Arrays use their
/// declared element marker; untyped arrays and object maps use `Object`.
/// Dictionaries convert keys as `String` and values as `Object`. Nested
/// collections can use nested adapters.
///
/// The adapter does not collect or allocate; iterator elements convert lazily.
/// Tuple elements are converted when [`Like::encode()`] is called. Values are
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
/// [`sys::FixedArrayOf`] accepts `Array([T; N])`, `Array(&[T; N])`, or a tuple
/// of the required arity. A different array length or tuple arity is rejected:
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
/// An exact-size iterator cannot supply a fixed-length argument, even when its
/// runtime length matches. This includes array iterators: their type retains
/// the original length after elements have been consumed:
///
/// ```compile_fail
/// use nvim_rpc::api_types::encode::Convert;
/// use nvim_rpc::mp::encode::primitive::ArrayIter;
/// use nvim_rpc::{api, Neovim, Window};
///
/// let mut nvim = Neovim::new(Vec::new());
/// let mut position = [1i64, 2].into_iter();
/// position.next();
/// nvim.notify::<api::NvimWinSetCursor>((Window::CURRENT, Convert(ArrayIter(position))))
///     .unwrap();
/// ```
///
/// The same restriction applies without `Convert`:
///
/// ```compile_fail
/// use nvim_rpc::mp::encode::primitive::ArrayIter;
/// use nvim_rpc::{api, Neovim, Window};
///
/// let mut nvim = Neovim::new(Vec::new());
/// let mut position = [1i64, 2].into_iter();
/// position.next();
/// nvim.notify::<api::NvimWinSetCursor>((Window::CURRENT, ArrayIter(position)))
///     .unwrap();
/// ```
///
/// Converting an iterator to `ArrayOf<B>` does not establish a fixed length:
///
/// ```compile_fail
/// use nvim_rpc::api_types::encode::{Convert, Like};
/// use nvim_rpc::mp::encode::primitive::ArrayIter;
/// use nvim_rpc::sys::types::{ArrayOf, Int};
/// use nvim_rpc::{api, Neovim, Window};
///
/// let position = <_ as Like<ArrayOf<Int>>>::encode(Convert(ArrayIter([1i64, 2].into_iter())));
/// let mut nvim = Neovim::new(Vec::new());
/// nvim.notify::<api::NvimWinSetCursor>((Window::CURRENT, position)).unwrap();
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
