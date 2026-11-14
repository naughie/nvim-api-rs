use super::super::Encode;
use super::{MAX_LEN, MAX_LEN_U32};
use super::{Primitive, Sealed, State, data};
use rmp::Marker;

/// Encodes an exactly sized collection of encodable key/value pairs as a map.
///
/// ```
/// use rmp_bytes::encode::primitive::Map;
/// use rmp_bytes::encode::WriterSyncExt as _;
///
/// let mut bytes = Vec::new();
/// bytes.write_primitive(Map([("x", 7u8), ("y", 9u8)])).consume();
/// assert_eq!(bytes, b"\x82\xa1x\x07\xa1y\x09");
/// ```
pub struct Map<M>(pub M);
/// Encodes an [`ExactSizeIterator`] yielding encodable key/value pairs.
pub struct MapIter<I>(pub I);

impl<M, K, V> Sealed for Map<M>
where
    K: Encode,
    V: Encode,
    M: IntoIterator<Item = (K, V)>,
    M::IntoIter: ExactSizeIterator,
{
}
impl<M, K, V> Primitive for Map<M>
where
    K: Encode,
    V: Encode,
    M: IntoIterator<Item = (K, V)>,
    M::IntoIter: ExactSizeIterator,
{
    type State = MapIterState<M::IntoIter, K, V>;

    fn into_state(self) -> Self::State {
        MapIter(self.0.into_iter()).into_state()
    }
}

impl<I, K, V> Sealed for MapIter<I>
where
    K: Encode,
    V: Encode,
    I: ExactSizeIterator<Item = (K, V)>,
{
}
impl<I, K, V> Primitive for MapIter<I>
where
    K: Encode,
    V: Encode,
    I: ExactSizeIterator<Item = (K, V)>,
{
    type State = MapIterState<I, K, V>;

    fn into_state(self) -> Self::State {
        MapIterState(Some(map_len(self.0.len())), Flatten(self.0, None, None, 0))
    }
}

pub struct MapIterState<I, K: Encode, V: Encode>(Option<LenType>, Flatten<I, K, V>);

fn map_len(len: usize) -> LenType {
    match len {
        0..=15 => LenType::Fix(data::marker(Marker::FixMap(len as u8))),
        0x10..=0xffff => LenType::U16(data::len(Marker::Map16, len as u16)),
        _ => LenType::U32(data::len(Marker::Map32, std::cmp::min(len, MAX_LEN) as u32)),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LenType {
    Fix([u8; 1]),
    U16([u8; 3]),
    U32([u8; 5]),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MapItem<K, V> {
    Marker(LenType),
    Key(K),
    Value(V),
}

impl<K: AsRef<[u8]>, V: AsRef<[u8]>> AsRef<[u8]> for MapItem<K, V> {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Marker(LenType::Fix(v)) => v,
            Self::Marker(LenType::U16(v)) => v,
            Self::Marker(LenType::U32(v)) => v,
            Self::Key(v) => v.as_ref(),
            Self::Value(v) => v.as_ref(),
        }
    }
}

pub struct Flatten<I, K: Encode, V: Encode>(I, Option<K::Encoder>, Option<V::Encoder>, u32);

enum FlattenItem<K: Encode, V: Encode> {
    Key(<K::Encoder as State>::Item),
    Value(<V::Encoder as State>::Item),
}

impl<I, K, V> Flatten<I, K, V>
where
    K: Encode,
    V: Encode,
    I: ExactSizeIterator<Item = (K, V)>,
{
    fn next(&mut self) -> Option<FlattenItem<K, V>> {
        loop {
            if let Some(key) = &mut self.1
                && let Some(ret) = key.next()
            {
                return Some(FlattenItem::Key(ret));
            }
            self.1 = None;

            if let Some(value) = &mut self.2
                && let Some(ret) = value.next()
            {
                return Some(FlattenItem::Value(ret));
            }
            self.2 = None;

            if self.3 == MAX_LEN_U32 {
                return None;
            }

            let (next_key, next_value) = self.0.next()?;
            self.3 += 1;
            self.1 = Some(next_key.into_encoder());
            self.2 = Some(next_value.into_encoder());
        }
    }
}

impl<I, K, V> State for MapIterState<I, K, V>
where
    K: Encode,
    V: Encode,
    I: ExactSizeIterator<Item = (K, V)>,
{
    type Item = MapItem<<K::Encoder as State>::Item, <V::Encoder as State>::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(marker) = self.0.take() {
            return Some(MapItem::Marker(marker));
        }
        let item = self.1.next()?;
        match item {
            FlattenItem::Key(key) => Some(MapItem::Key(key)),
            FlattenItem::Value(key) => Some(MapItem::Value(key)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::count_encoded_with_marker;
    use super::super::encode_to_vec;
    use super::super::{ArrayIter, Nil};
    use super::*;
    use rmp::Marker;
    use rmpv::Value;

    use std::io::Cursor;

    #[test]
    fn eq_encoded() {
        fn assert_impl<I, K, V>(values: I, expected: Vec<(Value, Value)>)
        where
            K: Primitive,
            V: Primitive,
            I: IntoIterator<Item = (K, V)>,
            I::IntoIter: ExactSizeIterator,
        {
            let values = encode_to_vec(MapIter(values.into_iter()));
            let mut buf = Vec::new();
            let mut wtr = Cursor::new(&mut buf);
            rmpv::encode::write_value(&mut wtr, &Value::Map(expected)).unwrap();
            assert_eq!(values, buf);
        }

        assert_impl([(0u8, 0u8); 0], vec![]);
        assert_impl(
            [("key0", 0u8), ("key1", 1u8), ("key2", 2u8)],
            vec![
                (Value::from("key0"), Value::from(0)),
                (Value::from("key1"), Value::from(1)),
                (Value::from("key2"), Value::from(2)),
            ],
        );
        assert_impl(
            [
                ("key0", "value0"),
                ("key1", "value1"),
                ("key2", "value2"),
                ("key3", "value3"),
            ],
            vec![
                (Value::from("key0"), Value::from("value0")),
                (Value::from("key1"), Value::from("value1")),
                (Value::from("key2"), Value::from("value2")),
                (Value::from("key3"), Value::from("value3")),
            ],
        );

        assert_impl(
            [
                ("key0", ArrayIter([1u8].iter().copied())),
                ("key1", ArrayIter([2u8, 3u8].iter().copied())),
                ("key2", ArrayIter([].iter().copied())),
                ("key3", ArrayIter([4u8, 5u8, 6u8].iter().copied())),
            ],
            vec![
                (Value::from("key0"), Value::Array(vec![Value::from(1u8)])),
                (
                    Value::from("key1"),
                    Value::Array(vec![Value::from(2u8), Value::from(3u8)]),
                ),
                (Value::from("key2"), Value::Array(vec![])),
                (
                    Value::from("key3"),
                    Value::Array(vec![Value::from(4u8), Value::from(5u8), Value::from(6u8)]),
                ),
            ],
        );

        assert_impl(
            [
                (
                    "key0",
                    MapIter([("inner0", ArrayIter([1u8].iter().copied()))].into_iter()),
                ),
                (
                    "key1",
                    MapIter([("inner1", ArrayIter([2u8, 3u8].iter().copied()))].into_iter()),
                ),
            ],
            vec![
                (
                    Value::from("key0"),
                    Value::Map(vec![(
                        Value::from("inner0"),
                        Value::Array(vec![Value::from(1u8)]),
                    )]),
                ),
                (
                    Value::from("key1"),
                    Value::Map(vec![(
                        Value::from("inner1"),
                        Value::Array(vec![Value::from(2u8), Value::from(3u8)]),
                    )]),
                ),
            ],
        );

        {
            let count = count_encoded_with_marker(
                MapIter((0..15).map(|_| (Nil, Nil))),
                &[Marker::FixMap(15).to_u8()],
            );
            assert_eq!(count, 31);
        }
        {
            let count = count_encoded_with_marker(
                MapIter((0..16).map(|_| (Nil, Nil))),
                &[Marker::Map16.to_u8(), 0x00, 0x10],
            );
            assert_eq!(count, 35);
        }
        {
            let count = count_encoded_with_marker(
                MapIter((0..0x10000).map(|_| (Nil, Nil))),
                &[Marker::Map32.to_u8(), 0x00, 0x01, 0x00, 0x00],
            );
            assert_eq!(count, 0x20000 + 5);
        }
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    #[ignore = "exercises u32::MAX-length iterators"]
    fn max_length_and_overflow() {
        {
            let count = count_encoded_with_marker(
                MapIter((0..MAX_LEN).map(|_| (Nil, Nil))),
                &[Marker::Map32.to_u8(), 0xff, 0xff, 0xff, 0xff],
            );
            assert_eq!(count, 2 * MAX_LEN + 5);
        }
        {
            let count = count_encoded_with_marker(
                MapIter((0..(MAX_LEN + 1)).map(|_| (Nil, Nil))),
                &[Marker::Map32.to_u8(), 0xff, 0xff, 0xff, 0xff],
            );
            assert_eq!(count, 2 * MAX_LEN + 5);
        }
    }
}
