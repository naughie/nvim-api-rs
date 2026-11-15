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

impl<A, F, B: BaseType> Like<sys::ArrayOf<B>> for Map<A, F>
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

impl<A, F> Like<sys::Array> for Map<A, F>
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

impl<A, F> Like<sys::Object> for Map<A, F>
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

impl<'s, A: ?Sized, F, B: BaseType> Like<sys::ArrayOf<B>> for MapRef<'s, A, F>
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

impl<'s, A: ?Sized, F> Like<sys::Array> for MapRef<'s, A, F>
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

impl<'s, A: ?Sized, F> Like<sys::Object> for MapRef<'s, A, F>
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

impl<T, F, B: BaseType, const N: usize> Like<sys::FixedArrayOf<B, N>> for Map<[T; N], F>
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

impl<T, F, B: BaseType, const N: usize> LikeBaseType<sys::FixedArrayOf<B, N>>
    for Array<Elements<Map<[T; N], F>, B>>
where
    Map<[T; N], F>: IntoIterator,
    <Map<[T; N], F> as IntoIterator>::IntoIter: ExactSizeIterator,
    <Map<[T; N], F> as IntoIterator>::Item: Like<B>,
{
}

impl<'s, T, F, B: BaseType, const N: usize> Like<sys::FixedArrayOf<B, N>> for Map<&'s [T; N], F>
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

impl<'s, T, F, B: BaseType, const N: usize> LikeBaseType<sys::FixedArrayOf<B, N>>
    for Array<Elements<Map<&'s [T; N], F>, B>>
where
    Map<&'s [T; N], F>: IntoIterator,
    <Map<&'s [T; N], F> as IntoIterator>::IntoIter: ExactSizeIterator,
    <Map<&'s [T; N], F> as IntoIterator>::Item: Like<B>,
{
}

impl<T, F, B: BaseType, const N: usize> Like<sys::FixedArrayOf<B, N>> for Map<Array<[T; N]>, F>
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

impl<T, F, B: BaseType, const N: usize> LikeBaseType<sys::FixedArrayOf<B, N>>
    for Array<Elements<Map<Array<[T; N]>, F>, B>>
where
    Map<Array<[T; N]>, F>: IntoIterator,
    <Map<Array<[T; N]>, F> as IntoIterator>::IntoIter: ExactSizeIterator,
    <Map<Array<[T; N]>, F> as IntoIterator>::Item: Like<B>,
{
}

impl<'s, T, F, B: BaseType, const N: usize> Like<sys::FixedArrayOf<B, N>>
    for Map<Array<&'s [T; N]>, F>
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

impl<'s, T, F, B: BaseType, const N: usize> LikeBaseType<sys::FixedArrayOf<B, N>>
    for Array<Elements<Map<Array<&'s [T; N]>, F>, B>>
where
    Map<Array<&'s [T; N]>, F>: IntoIterator,
    <Map<Array<&'s [T; N]>, F> as IntoIterator>::IntoIter: ExactSizeIterator,
    <Map<Array<&'s [T; N]>, F> as IntoIterator>::Item: Like<B>,
{
}

impl<'s, T, F, B: BaseType, const N: usize> Like<sys::FixedArrayOf<B, N>> for MapRef<'s, [T; N], F>
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

impl<'s, T, F, B: BaseType, const N: usize> LikeBaseType<sys::FixedArrayOf<B, N>>
    for Array<Elements<MapRef<'s, [T; N], F>, B>>
where
    MapRef<'s, [T; N], F>: IntoIterator,
    <MapRef<'s, [T; N], F> as IntoIterator>::IntoIter: ExactSizeIterator,
    <MapRef<'s, [T; N], F> as IntoIterator>::Item: Like<B>,
{
}

impl<'s, T, F, B: BaseType, const N: usize> Like<sys::FixedArrayOf<B, N>>
    for MapRef<'s, Array<[T; N]>, F>
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

impl<'s, T, F, B: BaseType, const N: usize> LikeBaseType<sys::FixedArrayOf<B, N>>
    for Array<Elements<MapRef<'s, Array<[T; N]>, F>, B>>
where
    MapRef<'s, Array<[T; N]>, F>: IntoIterator,
    <MapRef<'s, Array<[T; N]>, F> as IntoIterator>::IntoIter: ExactSizeIterator,
    <MapRef<'s, Array<[T; N]>, F> as IntoIterator>::Item: Like<B>,
{
}
