//! Signature traits and type markers for generated API descriptors.
//!
//! [`ApiFn`], [`ApiMethod`], and [`UiEvent`] expose names and signatures without
//! requiring descriptor values. Their associated types use the markers in this
//! module and [`crate::ext_types`]; tuples represent parameter lists.
//!
//! Markers describe API types; bindings select their Rust storage and encoding.
//!
//! [`Ref`] marks Lua-only function references. It has no RPC representation in
//! `nvim-rpc`.

use std::marker::PhantomData;

/// The full name and signature of a Neovim API function.
///
/// ```
/// use nvim_rpc_sys::{functions::NvimSetCurrentLine, types::{self, ApiFn}};
///
/// fn name<F: ApiFn<Params = (types::String,), Return = ()>>() -> &'static str {
///     F::NAME
/// }
/// assert_eq!(name::<NvimSetCurrentLine>(), "nvim_set_current_line");
/// ```
pub trait ApiFn {
    /// Parameter type markers in declaration order, including a method's receiver.
    type Params;
    /// The return type marker; `()` represents a void API return.
    type Return;
    /// The full API name, such as `"nvim_buf_get_lines"`.
    const NAME: &str;
}

/// A method signature with its handle receiver separated from its arguments.
///
/// Generated methods also implement [`ApiFn`], whose `NAME` retains the full
/// API prefix:
///
/// ```
/// use nvim_rpc_sys::functions::NvimBufLineCount;
/// use nvim_rpc_sys::types::{ApiFn, ApiMethod};
///
/// assert_eq!(<NvimBufLineCount as ApiFn>::NAME, "nvim_buf_line_count");
/// assert_eq!(<NvimBufLineCount as ApiMethod>::NAME, "line_count");
/// ```
pub trait ApiMethod {
    /// The receiver marker, such as [`crate::ext_types::Buffer`].
    type On;
    /// Parameter markers after removing the receiver from the API signature.
    type Params;
    /// The return type marker.
    type Return;
    /// The method name after removing the handle's prefix.
    const NAME: &str;
}

/// The name and argument markers of one UI event call.
///
/// A `grid_resize` call carries the grid ID, width, and height, each an Integer:
///
/// ```
/// use nvim_rpc_sys::{types::{self, UiEvent}, ui_events::GridResize};
///
/// fn grid_event<E: UiEvent<Params = (types::Int, types::Int, types::Int)>>()
///     -> &'static str
/// {
///     E::NAME
/// }
/// assert_eq!(grid_event::<GridResize>(), "grid_resize");
/// ```
pub trait UiEvent {
    /// Argument markers for one call of this event.
    type Params;
    /// The event name within a redraw notification, such as `"grid_resize"`.
    const NAME: &str;
}

/// The Neovim Nil type marker.
pub struct Nil;
/// The Neovim String type marker, independent of Rust text or byte storage.
pub struct String;
/// The Neovim Boolean type marker.
pub struct Bool;
/// The Neovim Integer type marker (signed 64-bit).
pub struct Int;
/// The Neovim Float type marker (double precision).
pub struct Float;
/// An array with no declared element type.
pub struct Array;
/// An array whose elements have marker `T`.
///
/// For example, [`crate::functions::NvimBufGetLines`] returns `ArrayOf<String>`.
pub struct ArrayOf<T>(PhantomData<T>);
/// An array of exactly `N` elements with marker `T`.
///
/// ```
/// use nvim_rpc_sys::{functions::NvimWinGetCursor, types::{self, ApiFn}};
///
/// fn cursor_result<F: ApiFn<Return = types::FixedArrayOf<types::Int, 2>>>() {}
/// cursor_result::<NvimWinGetCursor>();
/// ```
pub struct FixedArrayOf<T, const N: usize>(PhantomData<T>);
/// The Neovim Dictionary type marker.
pub struct Dict;
/// A marker for a Neovim API value whose concrete type is unspecified.
pub struct Object;
/// A value described by either marker `A` or marker `B`.
///
/// Nest this marker to describe more alternatives, for example
/// `OneOf<Int, OneOf<String, Array>>`. The branches describe possible values;
/// they do not add a discriminator or an array to the RPC value.
/// Order and nesting remain part of the Rust type; alternatives are not
/// normalized or implicitly reordered.
pub struct OneOf<A, B>(PhantomData<fn() -> (A, B)>);
/// A Lua function reference used by Lua-only APIs.
pub enum Ref {}
