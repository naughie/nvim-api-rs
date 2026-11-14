use super::super::{BaseType, Like, LikeBaseType, Map, MapRef, sys};
use super::Elements;

use mp::encode::primitive::Array;

use std::iter;

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
        impl<$($generic)*, B: BaseType, const N: usize> Like<sys::FixedArrayOf<B, N>> for $ty
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

        impl<$($generic)*, B: BaseType, const N: usize>
            LikeBaseType<sys::FixedArrayOf<B, N>> for Array<Elements<$ty, B>>
        where
            $ty: IntoIterator,
            <$ty as IntoIterator>::IntoIter: ExactSizeIterator,
            <$ty as IntoIterator>::Item: Like<B>,
        {}
    };
}

_impl_fixed!([T, F] Map<[T; N], F>);
_impl_fixed!(['s, T, F] Map<&'s [T; N], F>);
_impl_fixed!([T, F] Map<Array<[T; N]>, F>);
_impl_fixed!(['s, T, F] Map<Array<&'s [T; N]>, F>);
_impl_fixed!(['s, T, F] MapRef<'s, [T; N], F>);
_impl_fixed!(['s, T, F] MapRef<'s, Array<[T; N]>, F>);
