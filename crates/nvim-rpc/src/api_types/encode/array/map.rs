use super::{BaseType, Elements, Like, LikeBaseType, sys};

use mp::encode::primitive::Array;

use std::iter;

/// Maps an iterable's elements before converting them to the requested API type.
///
/// Consumes `A`, which may own or borrow its elements. `F` must implement `Fn`;
/// it is called once per visited element during encoding. The source iterator
/// must have an exact length. No intermediate collection is created.
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

impl<A, F, U> IntoIterator for Map<A, F>
where
    A: IntoIterator,
    F: Fn(A::Item) -> U,
{
    type Item = U;
    type IntoIter = iter::Map<A::IntoIter, F>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().map(self.1)
    }
}

/// Maps shared references to an iterable's elements without consuming the source.
///
/// `F` receives `&'s T` and may return a value borrowing that element. The
/// resulting encoding cannot outlive the source. Conversion remains lazy and
/// requires an exactly sized iterator, as with [`Map`].
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

impl<'s, A: ?Sized, T: 's, F, U> IntoIterator for MapRef<'s, A, F>
where
    &'s A: IntoIterator<Item = &'s T>,
    F: Fn(&'s T) -> U,
{
    type Item = U;
    type IntoIter = iter::Map<<&'s A as IntoIterator>::IntoIter, F>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().map(self.1)
    }
}

macro_rules! _impl_like {
    ([$($generic:tt)*] $ty:ty) => {
        impl<$($generic)*, B: BaseType> Like<sys::ArrayOf<B>> for $ty
        where
            Self: IntoIterator,
            <Self as IntoIterator>::IntoIter: ExactSizeIterator,
            <Self as IntoIterator>::Item: Like<B>,
        {
            type EncodeAs = Array<Elements<Self, B>>;

            fn encode(self) -> Self::EncodeAs {
                Array(Elements::new(self))
            }
        }

        impl<$($generic)*> Like<sys::Array> for $ty
        where
            Self: IntoIterator,
            <Self as IntoIterator>::IntoIter: ExactSizeIterator,
            <Self as IntoIterator>::Item: Like<sys::Object>,
        {
            type EncodeAs = Array<Elements<Self, sys::Object>>;

            fn encode(self) -> Self::EncodeAs {
                Array(Elements::new(self))
            }
        }

        impl<$($generic)*> Like<sys::Object> for $ty
        where
            Self: IntoIterator,
            <Self as IntoIterator>::IntoIter: ExactSizeIterator,
            <Self as IntoIterator>::Item: Like<sys::Object>,
        {
            type EncodeAs = Array<Elements<Self, sys::Object>>;

            fn encode(self) -> Self::EncodeAs {
                Array(Elements::new(self))
            }
        }
    };
}

_impl_like!([A, F] Map<A, F>);
_impl_like!(['s, A: ?Sized, F] MapRef<'s, A, F>);

macro_rules! _impl_fixed {
    ([$($generic:tt)*] $ty:ty) => {
        impl<$($generic)*, B: BaseType, U, const N: usize> Like<sys::FixedArrayOf<B, N>> for $ty
        where
            Self: IntoIterator<Item = U>,
            <Self as IntoIterator>::IntoIter: ExactSizeIterator,
            U: Like<B>,
        {
            type EncodeAs = Array<Elements<Self, B>>;

            fn encode(self) -> Self::EncodeAs {
                Array(Elements::new(self))
            }
        }

        impl<$($generic)*, B: BaseType, U, const N: usize>
            LikeBaseType<sys::FixedArrayOf<B, N>> for Array<Elements<$ty, B>>
        where
            $ty: IntoIterator<Item = U>,
            <$ty as IntoIterator>::IntoIter: ExactSizeIterator,
            U: Like<B>,
        {}
    };
}

_impl_fixed!([T, F] Map<[T; N], F>);
_impl_fixed!(['s, T, F] Map<&'s [T; N], F>);
_impl_fixed!([T, F] Map<Array<[T; N]>, F>);
_impl_fixed!(['s, T, F] Map<Array<&'s [T; N]>, F>);
_impl_fixed!(['s, T, F] MapRef<'s, [T; N], F>);
_impl_fixed!(['s, T, F] MapRef<'s, Array<[T; N]>, F>);
