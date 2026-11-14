use super::super::base::NonUnion;
use super::sys::OneOf;
use super::{BaseType, LikeBaseType, Untagged};

impl<M, L, R> LikeBaseType<M> for Untagged<L, R>
where
    M: NonUnion,
    L: LikeBaseType<M>,
    R: LikeBaseType<M>,
{
}

impl<A, B, L, R> LikeBaseType<OneOf<A, B>> for Untagged<L, R>
where
    A: BaseType,
    B: BaseType,
    L: LikeBaseType<A>,
    R: LikeBaseType<B>,
{
}

impl<'s, M, L, R> LikeBaseType<M> for &'s Untagged<L, R>
where
    M: NonUnion,
    &'s L: LikeBaseType<M>,
    &'s R: LikeBaseType<M>,
{
}

impl<'s, A, B, L, R> LikeBaseType<OneOf<A, B>> for &'s Untagged<L, R>
where
    A: BaseType,
    B: BaseType,
    &'s L: LikeBaseType<A>,
    &'s R: LikeBaseType<B>,
{
}
