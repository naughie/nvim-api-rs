use super::LikeBaseType;
use super::mp_prim;
use super::sys;

use mp_prim::Primitive;
use mp_prim::{Map, MapIter};

impl<M, K, V> LikeBaseType<sys::Dict> for Map<M>
where
    K: LikeBaseType<sys::String>,
    V: Primitive,
    M: IntoIterator<Item = (K, V)>,
    M::IntoIter: ExactSizeIterator,
{
}
impl<M, K, V> LikeBaseType<sys::Object> for Map<M>
where
    K: Primitive,
    V: Primitive,
    M: IntoIterator<Item = (K, V)>,
    M::IntoIter: ExactSizeIterator,
{
}

impl<I, K, V> LikeBaseType<sys::Dict> for MapIter<I>
where
    K: LikeBaseType<sys::String>,
    V: Primitive,
    I: ExactSizeIterator<Item = (K, V)>,
{
}
impl<I, K, V> LikeBaseType<sys::Object> for MapIter<I>
where
    K: Primitive,
    V: Primitive,
    I: ExactSizeIterator<Item = (K, V)>,
{
}
