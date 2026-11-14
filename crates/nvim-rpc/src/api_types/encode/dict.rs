use super::mp_prim;
use super::sys;
use super::{BaseType, Convert, Like, LikeBaseType};

use mp::encode::Encode;
use mp_prim::{Map, MapIter};

use std::marker::PhantomData;

impl<M, K, V> LikeBaseType<sys::Dict> for Map<M>
where
    K: LikeBaseType<sys::String>,
    V: Encode,
    M: IntoIterator<Item = (K, V)>,
    M::IntoIter: ExactSizeIterator,
{
}
impl<M, K, V> LikeBaseType<sys::Object> for Map<M>
where
    K: Encode,
    V: Encode,
    M: IntoIterator<Item = (K, V)>,
    M::IntoIter: ExactSizeIterator,
{
}

impl<I, K, V> LikeBaseType<sys::Dict> for MapIter<I>
where
    K: LikeBaseType<sys::String>,
    V: Encode,
    I: ExactSizeIterator<Item = (K, V)>,
{
}
impl<I, K, V> LikeBaseType<sys::Object> for MapIter<I>
where
    K: Encode,
    V: Encode,
    I: ExactSizeIterator<Item = (K, V)>,
{
}

impl<M, K, V> Like<sys::Dict> for Convert<Map<M>>
where
    K: Like<sys::String>,
    V: Like<sys::Object>,
    M: IntoIterator<Item = (K, V)>,
    M::IntoIter: ExactSizeIterator,
{
    type EncodeAs = Map<Entries<M, sys::String, sys::Object>>;

    fn encode(self) -> Self::EncodeAs {
        Map(Entries(self.0.0, PhantomData))
    }
}

impl<M, K, V> Like<sys::Object> for Convert<Map<M>>
where
    K: Like<sys::Object>,
    V: Like<sys::Object>,
    M: IntoIterator<Item = (K, V)>,
    M::IntoIter: ExactSizeIterator,
{
    type EncodeAs = Map<Entries<M, sys::Object, sys::Object>>;

    fn encode(self) -> Self::EncodeAs {
        Map(Entries(self.0.0, PhantomData))
    }
}

impl<I, K, V> Like<sys::Dict> for Convert<MapIter<I>>
where
    K: Like<sys::String>,
    V: Like<sys::Object>,
    I: ExactSizeIterator<Item = (K, V)>,
{
    type EncodeAs = Map<Entries<I, sys::String, sys::Object>>;

    fn encode(self) -> Self::EncodeAs {
        Map(Entries(self.0.0, PhantomData))
    }
}

impl<I, K, V> Like<sys::Object> for Convert<MapIter<I>>
where
    K: Like<sys::Object>,
    V: Like<sys::Object>,
    I: ExactSizeIterator<Item = (K, V)>,
{
    type EncodeAs = Map<Entries<I, sys::Object, sys::Object>>;

    fn encode(self) -> Self::EncodeAs {
        Map(Entries(self.0.0, PhantomData))
    }
}

pub struct Entries<M, K, V>(M, PhantomData<fn() -> (K, V)>);

impl<M, K, V, A, B> IntoIterator for Entries<M, K, V>
where
    K: BaseType,
    V: BaseType,
    A: Like<K>,
    B: Like<V>,
    M: IntoIterator<Item = (A, B)>,
{
    type Item = (A::EncodeAs, B::EncodeAs);
    type IntoIter = EncodeEntries<M::IntoIter, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        EncodeEntries(self.0.into_iter(), PhantomData)
    }
}

pub struct EncodeEntries<I, K, V>(I, PhantomData<fn() -> (K, V)>);

impl<I, K, V, A, B> Iterator for EncodeEntries<I, K, V>
where
    K: BaseType,
    V: BaseType,
    A: Like<K>,
    B: Like<V>,
    I: Iterator<Item = (A, B)>,
{
    type Item = (A::EncodeAs, B::EncodeAs);

    fn next(&mut self) -> Option<Self::Item> {
        let (key, value) = self.0.next()?;
        Some((Like::encode(key), Like::encode(value)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<I, K, V, A, B> ExactSizeIterator for EncodeEntries<I, K, V>
where
    K: BaseType,
    V: BaseType,
    A: Like<K>,
    B: Like<V>,
    I: ExactSizeIterator<Item = (A, B)>,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}
