//! Argument and result representations for Neovim's type markers.
//!
//! `sys::types::String` describes an API type; it does not select Rust's
//! `String` as storage. [`encode::Like`] selects an input representation,
//! [`Params`] combines arguments, and [`Decode`] selects the output type.

pub mod encode;

pub mod decode;
pub use decode::{Decode, DecodeError, DecodedArray};

use encode::Like;
use mp::encode::primitive::ArrayLike;

/// Encodes an argument list matching an API's parameter markers.
///
/// Tuples through arity 15 are implemented by applying [`Like`] to each
/// position. For example, `(Buffer, i64, i64, bool, &[&str])` can represent
/// `nvim_buf_set_lines` parameters. [`crate::Neovim::request()`] checks this
/// relationship against the selected descriptor's `ApiFn::Params`.
pub trait Params<Signature> {
    /// The MessagePack array containing the converted arguments.
    type Encoded: ArrayLike;

    /// Consumes the arguments and converts each value to its wire representation.
    fn encode(self) -> Self::Encoded;
}

impl Params<()> for () {
    type Encoded = ();

    fn encode(self) -> Self::Encoded {}
}

macro_rules! _impl_params {
    ($($marker:ident $value:ident $idx:tt),+) => {
        impl<$($marker: BaseType, $value: Like<$marker>),+>
            Params<($($marker,)+)> for ($($value,)+)
        {
            type Encoded = ($($value::EncodeAs,)+);

            fn encode(self) -> Self::Encoded {
                ($(Like::encode(self.$idx),)+)
            }
        }
    };
}

_impl_params!(M0 A0 0);
_impl_params!(M0 A0 0, M1 A1 1);
_impl_params!(M0 A0 0, M1 A1 1, M2 A2 2);
_impl_params!(M0 A0 0, M1 A1 1, M2 A2 2, M3 A3 3);
_impl_params!(M0 A0 0, M1 A1 1, M2 A2 2, M3 A3 3, M4 A4 4);
_impl_params!(M0 A0 0, M1 A1 1, M2 A2 2, M3 A3 3, M4 A4 4, M5 A5 5);
_impl_params!(M0 A0 0, M1 A1 1, M2 A2 2, M3 A3 3, M4 A4 4, M5 A5 5, M6 A6 6);
_impl_params!(M0 A0 0, M1 A1 1, M2 A2 2, M3 A3 3, M4 A4 4, M5 A5 5, M6 A6 6, M7 A7 7);
_impl_params!(M0 A0 0, M1 A1 1, M2 A2 2, M3 A3 3, M4 A4 4, M5 A5 5, M6 A6 6, M7 A7 7, M8 A8 8);
_impl_params!(M0 A0 0, M1 A1 1, M2 A2 2, M3 A3 3, M4 A4 4, M5 A5 5, M6 A6 6, M7 A7 7, M8 A8 8, M9 A9 9);
_impl_params!(M0 A0 0, M1 A1 1, M2 A2 2, M3 A3 3, M4 A4 4, M5 A5 5, M6 A6 6, M7 A7 7, M8 A8 8, M9 A9 9, M10 A10 10);
_impl_params!(M0 A0 0, M1 A1 1, M2 A2 2, M3 A3 3, M4 A4 4, M5 A5 5, M6 A6 6, M7 A7 7, M8 A8 8, M9 A9 9, M10 A10 10, M11 A11 11);
_impl_params!(M0 A0 0, M1 A1 1, M2 A2 2, M3 A3 3, M4 A4 4, M5 A5 5, M6 A6 6, M7 A7 7, M8 A8 8, M9 A9 9, M10 A10 10, M11 A11 11, M12 A12 12);
_impl_params!(M0 A0 0, M1 A1 1, M2 A2 2, M3 A3 3, M4 A4 4, M5 A5 5, M6 A6 6, M7 A7 7, M8 A8 8, M9 A9 9, M10 A10 10, M11 A11 11, M12 A12 12, M13 A13 13);
_impl_params!(M0 A0 0, M1 A1 1, M2 A2 2, M3 A3 3, M4 A4 4, M5 A5 5, M6 A6 6, M7 A7 7, M8 A8 8, M9 A9 9, M10 A10 10, M11 A11 11, M12 A12 12, M13 A13 13, M14 A14 14);

use nvim_rpc_sys::types as sys;

pub use base::BaseType;
mod base {
    use super::sys::*;
    use nvim_rpc_sys::ext_types::{Buffer, Tabpage, Window};

    use _priv::Sealed;
    mod _priv {
        pub trait Sealed {}
    }

    /// A Neovim type marker, such as `Int`, `ArrayOf<String>`, or `Buffer`.
    ///
    /// `OneOf<A, B>` composes alternatives recursively. The vocabulary is
    /// sealed; applications customize their Rust representations through
    /// `Like` and `Decode`.
    pub trait BaseType: Sealed {}

    // Separates non-union markers from OneOf for recursive compatibility checks.
    pub(super) trait NonUnion: BaseType {}

    macro_rules! _impl_base_type {
        ($($ty:ty),+ $(,)?) => {
            $(
                impl Sealed for $ty {}
                impl BaseType for $ty {}
                impl NonUnion for $ty {}
            )+
        };
    }

    _impl_base_type!(
        Buffer, Window, Tabpage, Nil, String, Bool, Int, Float, Array, Dict, Object
    );

    impl<T: BaseType> Sealed for ArrayOf<T> {}
    impl<T: BaseType> BaseType for ArrayOf<T> {}
    impl<T: BaseType> NonUnion for ArrayOf<T> {}

    impl<T: BaseType, const N: usize> Sealed for FixedArrayOf<T, N> {}
    impl<T: BaseType, const N: usize> BaseType for FixedArrayOf<T, N> {}
    impl<T: BaseType, const N: usize> NonUnion for FixedArrayOf<T, N> {}

    impl<A: BaseType, B: BaseType> Sealed for OneOf<A, B> {}
    impl<A: BaseType, B: BaseType> BaseType for OneOf<A, B> {}
}
