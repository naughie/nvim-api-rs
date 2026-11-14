use super::{BaseType, Like, LikeBaseType};
use super::{mp_prim, sys};

use mp_prim::Primitive;
use mp_prim::{Array, ArrayIter};

use std::array::IntoIter as ArrayIntoIter;

impl<T, A> LikeBaseType<sys::Array> for Array<A>
where
    T: Primitive,
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
    &'s T: LikeBaseType<B>,
{
    type EncodeAs = Array<Self>;

    fn encode(self) -> Self::EncodeAs {
        Array(self)
    }
}

impl<'s, B: BaseType, T, const N: usize> Like<sys::ArrayOf<B>> for &'s [T; N]
where
    &'s T: LikeBaseType<B>,
{
    type EncodeAs = Array<Self>;

    fn encode(self) -> Self::EncodeAs {
        Array(self)
    }
}

impl<'s, B: BaseType, T, const N: usize> Like<sys::FixedArrayOf<B, N>> for &'s [T; N]
where
    &'s T: LikeBaseType<B>,
{
    type EncodeAs = Array<Self>;

    fn encode(self) -> Self::EncodeAs {
        Array(self)
    }
}

impl<T, A> LikeBaseType<sys::Object> for Array<A>
where
    T: Primitive,
    A: IntoIterator<Item = T>,
    A::IntoIter: ExactSizeIterator,
{
}

impl<T, I> LikeBaseType<sys::Array> for ArrayIter<I>
where
    T: Primitive,
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
    T: Primitive,
    I: ExactSizeIterator<Item = T>,
{
}

impl LikeBaseType<sys::Array> for () {}
impl<B: BaseType> LikeBaseType<sys::ArrayOf<B>> for () {}
impl<B: BaseType> LikeBaseType<sys::FixedArrayOf<B, 0>> for () {}

macro_rules! _impl_like_base_for_tuple {
    ($num:expr, $($ty:ident, )+) => {
        impl<$( $ty: Primitive, )+> LikeBaseType<sys::Array> for ( $( $ty, )+ ) {}

        impl<B: BaseType, $( $ty: LikeBaseType<B>, )+>
            LikeBaseType<sys::ArrayOf<B>> for ( $( $ty, )+ ) {}

        impl<B: BaseType, $( $ty: LikeBaseType<B>, )+>
            LikeBaseType<sys::FixedArrayOf<B, $num>> for ( $( $ty, )+ ) {}

        impl<$( $ty: Primitive, )+> LikeBaseType<sys::Object> for ( $( $ty, )+ ) {}
    };
}

_impl_like_base_for_tuple! {
    1, T1,
}
_impl_like_base_for_tuple! {
    2, T1, T2,
}
_impl_like_base_for_tuple! {
    3, T1, T2, T3,
}
_impl_like_base_for_tuple! {
    4, T1, T2, T3, T4,
}
_impl_like_base_for_tuple! {
    5, T1, T2, T3, T4, T5,
}
_impl_like_base_for_tuple! {
    6, T1, T2, T3, T4, T5, T6,
}
_impl_like_base_for_tuple! {
    7, T1, T2, T3, T4, T5, T6, T7,
}
_impl_like_base_for_tuple! {
    8, T1, T2, T3, T4, T5, T6, T7, T8,
}
_impl_like_base_for_tuple! {
    9, T1, T2, T3, T4, T5, T6, T7, T8, T9,
}
_impl_like_base_for_tuple! {
    10, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10,
}
_impl_like_base_for_tuple! {
    11, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11,
}
_impl_like_base_for_tuple! {
    12, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12,
}
_impl_like_base_for_tuple! {
    13, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13,
}
_impl_like_base_for_tuple! {
    14, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14,
}
_impl_like_base_for_tuple! {
    15, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15,
}
