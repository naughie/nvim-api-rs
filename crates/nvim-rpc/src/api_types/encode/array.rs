use super::{BaseType, Convert, Like, LikeBaseType};
use super::{mp_prim, sys};

use mp::encode::Encode;
use mp_prim::{Array, ArrayIter};

use std::array::IntoIter as ArrayIntoIter;
use std::marker::PhantomData;

mod map;
pub use map::{Map, MapRef};

pub struct Elements<A, B>(A, PhantomData<fn() -> B>);

impl<A: Clone, B> Clone for Elements<A, B> {
    fn clone(&self) -> Self {
        Self::new(self.0.clone())
    }
}

impl<A: Copy, B> Copy for Elements<A, B> {}

impl<A: PartialEq, B> PartialEq for Elements<A, B> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<A: Eq, B> Eq for Elements<A, B> {}

impl<A, B> Elements<A, B> {
    fn new(values: A) -> Self {
        Self(values, PhantomData)
    }
}

impl<A, B> IntoIterator for Elements<A, B>
where
    B: BaseType,
    A: IntoIterator,
    A::Item: Like<B>,
{
    type Item = <A::Item as Like<B>>::EncodeAs;
    type IntoIter = EncodeIter<A::IntoIter, B>;

    fn into_iter(self) -> Self::IntoIter {
        EncodeIter(self.0.into_iter(), PhantomData)
    }
}

pub struct EncodeIter<I, B>(I, PhantomData<fn() -> B>);

impl<I, B> Iterator for EncodeIter<I, B>
where
    B: BaseType,
    I: Iterator,
    I::Item: Like<B>,
{
    type Item = <I::Item as Like<B>>::EncodeAs;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Like::encode)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<I, B> ExactSizeIterator for EncodeIter<I, B>
where
    B: BaseType,
    I: ExactSizeIterator,
    I::Item: Like<B>,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

macro_rules! _impl_convert_array {
    ($wrapper:ident; $($bounds:tt)*) => {
        impl<B, A> Like<sys::ArrayOf<B>> for Convert<$wrapper<A>>
        where
            B: BaseType,
            A: IntoIterator,
            <A as IntoIterator>::Item: Like<B>,
            A::IntoIter: ExactSizeIterator,
            $($bounds)*
        {
            type EncodeAs = Array<Elements<A, B>>;

            fn encode(self) -> Self::EncodeAs {
                Array(Elements::new(self.0.0))
            }
        }

        impl<A> Like<sys::Array> for Convert<$wrapper<A>>
        where
            A: IntoIterator,
            <A as IntoIterator>::Item: Like<sys::Object>,
            A::IntoIter: ExactSizeIterator,
            $($bounds)*
        {
            type EncodeAs = Array<Elements<A, sys::Object>>;

            fn encode(self) -> Self::EncodeAs {
                Array(Elements::new(self.0.0))
            }
        }

        impl<A> Like<sys::Object> for Convert<$wrapper<A>>
        where
            A: IntoIterator,
            <A as IntoIterator>::Item: Like<sys::Object>,
            A::IntoIter: ExactSizeIterator,
            $($bounds)*
        {
            type EncodeAs = Array<Elements<A, sys::Object>>;

            fn encode(self) -> Self::EncodeAs {
                Array(Elements::new(self.0.0))
            }
        }
    };
}

_impl_convert_array!(Array;);
_impl_convert_array!(ArrayIter; A: ExactSizeIterator,);

impl<B: BaseType, T: Like<B>, const N: usize> Like<sys::FixedArrayOf<B, N>>
    for Convert<Array<[T; N]>>
{
    type EncodeAs = Array<Elements<[T; N], B>>;

    fn encode(self) -> Self::EncodeAs {
        Array(Elements::new(self.0.0))
    }
}

impl<'s, B: BaseType, T, const N: usize> Like<sys::FixedArrayOf<B, N>>
    for Convert<Array<&'s [T; N]>>
where
    &'s T: Like<B>,
{
    type EncodeAs = Array<Elements<&'s [T; N], B>>;

    fn encode(self) -> Self::EncodeAs {
        Array(Elements::new(self.0.0))
    }
}

impl<B: BaseType, T: Like<B>, const N: usize> Like<sys::FixedArrayOf<B, N>>
    for Convert<ArrayIter<ArrayIntoIter<T, N>>>
{
    type EncodeAs = Array<Elements<ArrayIntoIter<T, N>, B>>;

    fn encode(self) -> Self::EncodeAs {
        Array(Elements::new(self.0.0))
    }
}

impl<B: BaseType, T: Like<B>, const N: usize> LikeBaseType<sys::FixedArrayOf<B, N>>
    for Array<Elements<[T; N], B>>
{
}

impl<'s, B: BaseType, T, const N: usize> LikeBaseType<sys::FixedArrayOf<B, N>>
    for Array<Elements<&'s [T; N], B>>
where
    &'s T: Like<B>,
{
}

impl<B: BaseType, T: Like<B>, const N: usize> LikeBaseType<sys::FixedArrayOf<B, N>>
    for Array<Elements<ArrayIntoIter<T, N>, B>>
{
}

impl<T, A> LikeBaseType<sys::Array> for Array<A>
where
    T: Encode,
    A: IntoIterator<Item = T>,
    A::IntoIter: ExactSizeIterator,
{
}
impl<B, T, A> LikeBaseType<sys::ArrayOf<B>> for Array<A>
where
    B: BaseType,
    T: LikeBaseType<B>,
    A: IntoIterator<Item = T>,
    A::IntoIter: ExactSizeIterator,
{
}

impl<B, T, const N: usize> LikeBaseType<sys::FixedArrayOf<B, N>> for Array<[T; N]>
where
    B: BaseType,
    T: LikeBaseType<B>,
{
}

impl<'s, B: BaseType, T, const N: usize> LikeBaseType<sys::FixedArrayOf<B, N>> for Array<&'s [T; N]> where
    &'s T: LikeBaseType<B>
{
}

impl<'s, B: BaseType, T> Like<sys::ArrayOf<B>> for &'s [T]
where
    &'s T: Like<B>,
{
    type EncodeAs = Array<Elements<Self, B>>;

    fn encode(self) -> Self::EncodeAs {
        Array(Elements::new(self))
    }
}

impl<'s, B: BaseType, T, const N: usize> Like<sys::ArrayOf<B>> for &'s [T; N]
where
    &'s T: Like<B>,
{
    type EncodeAs = Array<Elements<Self, B>>;

    fn encode(self) -> Self::EncodeAs {
        Array(Elements::new(self))
    }
}

impl<'s, B: BaseType, T, const N: usize> Like<sys::FixedArrayOf<B, N>> for &'s [T; N]
where
    &'s T: Like<B>,
{
    type EncodeAs = Array<Elements<Self, B>>;

    fn encode(self) -> Self::EncodeAs {
        Array(Elements::new(self))
    }
}

impl<T, A> LikeBaseType<sys::Object> for Array<A>
where
    T: Encode,
    A: IntoIterator<Item = T>,
    A::IntoIter: ExactSizeIterator,
{
}

impl<T, I> LikeBaseType<sys::Array> for ArrayIter<I>
where
    T: Encode,
    I: ExactSizeIterator<Item = T>,
{
}
impl<B, T, I> LikeBaseType<sys::ArrayOf<B>> for ArrayIter<I>
where
    B: BaseType,
    T: LikeBaseType<B>,
    I: ExactSizeIterator<Item = T>,
{
}

impl<B, T, const N: usize> LikeBaseType<sys::FixedArrayOf<B, N>> for ArrayIter<ArrayIntoIter<T, N>>
where
    B: BaseType,
    T: LikeBaseType<B>,
{
}

impl<T, I> LikeBaseType<sys::Object> for ArrayIter<I>
where
    T: Encode,
    I: ExactSizeIterator<Item = T>,
{
}

impl LikeBaseType<sys::Array> for () {}
impl<B: BaseType> LikeBaseType<sys::ArrayOf<B>> for () {}
impl<B: BaseType> LikeBaseType<sys::FixedArrayOf<B, 0>> for () {}

impl Like<sys::Array> for Convert<()> {
    type EncodeAs = ();

    fn encode(self) -> Self::EncodeAs {}
}

impl<B: BaseType> Like<sys::ArrayOf<B>> for Convert<()> {
    type EncodeAs = ();

    fn encode(self) -> Self::EncodeAs {}
}

impl<B: BaseType> Like<sys::FixedArrayOf<B, 0>> for Convert<()> {
    type EncodeAs = ();

    fn encode(self) -> Self::EncodeAs {}
}

macro_rules! _impl_like_base_for_tuple {
    ($num:expr, $($ty:ident $idx:tt, )+) => {
        impl<$( $ty: Encode, )+> LikeBaseType<sys::Array> for ( $( $ty, )+ ) {}

        impl<B: BaseType, $( $ty: LikeBaseType<B>, )+>
            LikeBaseType<sys::ArrayOf<B>> for ( $( $ty, )+ ) {}

        impl<B: BaseType, $( $ty: LikeBaseType<B>, )+>
            LikeBaseType<sys::FixedArrayOf<B, $num>> for ( $( $ty, )+ ) {}

        impl<$( $ty: Encode, )+> LikeBaseType<sys::Object> for ( $( $ty, )+ ) {}

        impl<B: BaseType, $( $ty: Like<B>, )+>
            Like<sys::ArrayOf<B>> for Convert<( $( $ty, )+ )>
        {
            type EncodeAs = ( $( $ty::EncodeAs, )+ );

            fn encode(self) -> Self::EncodeAs {
                ( $( Like::encode(self.0.$idx), )+ )
            }
        }

        impl<B: BaseType, $( $ty: Like<B>, )+>
            Like<sys::FixedArrayOf<B, $num>> for Convert<( $( $ty, )+ )>
        {
            type EncodeAs = ( $( $ty::EncodeAs, )+ );

            fn encode(self) -> Self::EncodeAs {
                ( $( Like::encode(self.0.$idx), )+ )
            }
        }

        impl<$( $ty: Like<sys::Object>, )+> Like<sys::Array> for Convert<( $( $ty, )+ )> {
            type EncodeAs = ( $( $ty::EncodeAs, )+ );

            fn encode(self) -> Self::EncodeAs {
                ( $( Like::encode(self.0.$idx), )+ )
            }
        }

        impl<$( $ty: Like<sys::Object>, )+> Like<sys::Object> for Convert<( $( $ty, )+ )> {
            type EncodeAs = ( $( $ty::EncodeAs, )+ );

            fn encode(self) -> Self::EncodeAs {
                ( $( Like::encode(self.0.$idx), )+ )
            }
        }
    };
}

_impl_like_base_for_tuple! {
    1, T1 0,
}
_impl_like_base_for_tuple! {
    2, T1 0, T2 1,
}
_impl_like_base_for_tuple! {
    3, T1 0, T2 1, T3 2,
}
_impl_like_base_for_tuple! {
    4, T1 0, T2 1, T3 2, T4 3,
}
_impl_like_base_for_tuple! {
    5, T1 0, T2 1, T3 2, T4 3, T5 4,
}
_impl_like_base_for_tuple! {
    6, T1 0, T2 1, T3 2, T4 3, T5 4, T6 5,
}
_impl_like_base_for_tuple! {
    7, T1 0, T2 1, T3 2, T4 3, T5 4, T6 5, T7 6,
}
_impl_like_base_for_tuple! {
    8, T1 0, T2 1, T3 2, T4 3, T5 4, T6 5, T7 6, T8 7,
}
_impl_like_base_for_tuple! {
    9, T1 0, T2 1, T3 2, T4 3, T5 4, T6 5, T7 6, T8 7, T9 8,
}
_impl_like_base_for_tuple! {
    10, T1 0, T2 1, T3 2, T4 3, T5 4, T6 5, T7 6, T8 7, T9 8, T10 9,
}
_impl_like_base_for_tuple! {
    11, T1 0, T2 1, T3 2, T4 3, T5 4, T6 5, T7 6, T8 7, T9 8, T10 9, T11 10,
}
_impl_like_base_for_tuple! {
    12, T1 0, T2 1, T3 2, T4 3, T5 4, T6 5, T7 6, T8 7, T9 8, T10 9, T11 10, T12 11,
}
_impl_like_base_for_tuple! {
    13, T1 0, T2 1, T3 2, T4 3, T5 4, T6 5, T7 6, T8 7, T9 8, T10 9, T11 10, T12 11, T13 12,
}
_impl_like_base_for_tuple! {
    14, T1 0, T2 1, T3 2, T4 3, T5 4, T6 5, T7 6, T8 7, T9 8, T10 9, T11 10, T12 11, T13 12, T14 13,
}
_impl_like_base_for_tuple! {
    15, T1 0, T2 1, T3 2, T4 3, T5 4, T6 5, T7 6, T8 7, T9 8, T10 9, T11 10, T12 11, T13 12, T14 13, T15 14,
}
