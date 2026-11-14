use super::super::Encode;
use super::ArrayLike;
use super::{MAX_LEN, MAX_LEN_U32};
use super::{Primitive, Sealed, State, data};
use rmp::Marker;

/// Encodes an exactly sized collection as a MessagePack array.
///
/// `Array([1u8, 2, 3])` consumes an array; `Array(&["a", "b"][..])`
/// borrows a string slice. Each yielded element must implement [`Encode`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Array<A>(pub A);
/// Encodes an [`ExactSizeIterator`] without collecting its elements.
///
/// ```
/// use rmp_bytes::encode::primitive::ArrayIter;
/// use rmp_bytes::encode::WriterSyncExt as _;
///
/// let mut bytes = Vec::new();
/// bytes.write_primitive(ArrayIter((0u8..3).map(|n| n + 1))).consume();
/// assert_eq!(bytes, b"\x93\x01\x02\x03");
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ArrayIter<I>(pub I);

impl<A, T> Sealed for Array<A>
where
    T: Encode,
    A: IntoIterator<Item = T>,
    A::IntoIter: ExactSizeIterator,
{
}
impl<A, T> Primitive for Array<A>
where
    T: Encode,
    A: IntoIterator<Item = T>,
    A::IntoIter: ExactSizeIterator,
{
    type State = ArrayIterState<A::IntoIter, T>;

    fn into_state(self) -> Self::State {
        ArrayIter(self.0.into_iter()).into_state()
    }
}
impl<A, T> ArrayLike for Array<A>
where
    T: Encode,
    A: IntoIterator<Item = T>,
    A::IntoIter: ExactSizeIterator,
{
}

impl<T: Encode, I: ExactSizeIterator<Item = T>> Sealed for ArrayIter<I> {}
impl<T: Encode, I: ExactSizeIterator<Item = T>> Primitive for ArrayIter<I> {
    type State = ArrayIterState<I, T>;

    fn into_state(self) -> Self::State {
        ArrayIterState(Some(array_len(self.0.len())), Flatten(self.0, None, 0))
    }
}
impl<T: Encode, I: ExactSizeIterator<Item = T>> ArrayLike for ArrayIter<I> {}

pub struct ArrayIterState<I, T: Encode>(Option<LenType>, Flatten<I, T>);

fn array_len(len: usize) -> LenType {
    match len {
        0..=15 => LenType::Fix(data::marker(Marker::FixArray(len as u8))),
        0x10..=0xffff => LenType::U16(data::len(Marker::Array16, len as u16)),
        _ => LenType::U32(data::len(
            Marker::Array32,
            std::cmp::min(len, MAX_LEN) as u32,
        )),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LenType {
    Fix([u8; 1]),
    U16([u8; 3]),
    U32([u8; 5]),
}

pub enum ArrayItem<T> {
    Marker(LenType),
    Item(T),
}

impl<T: AsRef<[u8]>> AsRef<[u8]> for ArrayItem<T> {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Marker(LenType::Fix(v)) => v,
            Self::Marker(LenType::U16(v)) => v,
            Self::Marker(LenType::U32(v)) => v,
            Self::Item(v) => v.as_ref(),
        }
    }
}

pub struct Flatten<I, T: Encode>(I, Option<T::Encoder>, u32);

impl<T: Encode, I: ExactSizeIterator<Item = T>> Flatten<I, T> {
    fn next(&mut self) -> Option<<T::Encoder as State>::Item> {
        loop {
            if let Some(state) = &mut self.1
                && let Some(ret) = state.next()
            {
                return Some(ret);
            }
            self.1 = None;

            if self.2 == MAX_LEN_U32 {
                return None;
            }

            let next_state = self.0.next()?;
            self.2 += 1;
            self.1 = Some(next_state.into_encoder());
        }
    }
}

impl<T: Encode, I: ExactSizeIterator<Item = T>> State for ArrayIterState<I, T> {
    type Item = ArrayItem<<T::Encoder as State>::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(marker) = self.0.take() {
            return Some(ArrayItem::Marker(marker));
        }
        let item = self.1.next()?;
        Some(ArrayItem::Item(item))
    }
}

#[cfg(test)]
mod tests {
    use super::super::Nil;
    use super::super::count_encoded_with_marker;
    use super::super::encode_to_vec;
    use super::*;
    use rmp::Marker;
    use rmpv::Value;

    use std::io::Cursor;

    #[test]
    fn eq_encoded() {
        fn assert_impl<I, T>(values: I, expected: Vec<Value>)
        where
            T: Primitive,
            I: IntoIterator<Item = T>,
            I::IntoIter: ExactSizeIterator,
        {
            let values = encode_to_vec(ArrayIter(values.into_iter()));
            let mut buf = Vec::new();
            let mut wtr = Cursor::new(&mut buf);
            rmpv::encode::write_value(&mut wtr, &Value::Array(expected)).unwrap();
            assert_eq!(values, buf);
        }

        assert_impl([0u8; 0], vec![]);
        assert_impl(
            [0u8, 1u8, 2u8],
            vec![Value::from(0), Value::from(1), Value::from(2)],
        );
        assert_impl(
            ["fizz", "baz", "fizzbuzz", "number"],
            vec![
                Value::from("fizz"),
                Value::from("baz"),
                Value::from("fizzbuzz"),
                Value::from("number"),
            ],
        );

        assert_impl(
            [
                ArrayIter([1u8].iter().copied()),
                ArrayIter([2u8, 3u8].iter().copied()),
                ArrayIter([].iter().copied()),
                ArrayIter([4u8, 5u8, 6u8].iter().copied()),
            ],
            vec![
                Value::Array(vec![Value::from(1u8)]),
                Value::Array(vec![Value::from(2u8), Value::from(3u8)]),
                Value::Array(vec![]),
                Value::Array(vec![Value::from(4u8), Value::from(5u8), Value::from(6u8)]),
            ],
        );

        {
            let count = count_encoded_with_marker(
                ArrayIter((0..15).map(|_| Nil)),
                &[Marker::FixArray(15).to_u8()],
            );
            assert_eq!(count, 16);
        }
        {
            let count = count_encoded_with_marker(
                ArrayIter((0..16).map(|_| Nil)),
                &[Marker::Array16.to_u8(), 0x00, 0x10],
            );
            assert_eq!(count, 19);
        }
        {
            let count = count_encoded_with_marker(
                ArrayIter((0..0x10000).map(|_| Nil)),
                &[Marker::Array32.to_u8(), 0x00, 0x01, 0x00, 0x00],
            );
            assert_eq!(count, 0x10000 + 5);
        }
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    #[ignore = "exercises u32::MAX-length iterators"]
    fn max_length_and_overflow() {
        {
            let count = count_encoded_with_marker(
                ArrayIter((0..MAX_LEN).map(|_| Nil)),
                &[Marker::Array32.to_u8(), 0xff, 0xff, 0xff, 0xff],
            );
            assert_eq!(count, MAX_LEN + 5);
        }
        {
            let count = count_encoded_with_marker(
                ArrayIter((0..(MAX_LEN + 1)).map(|_| Nil)),
                &[Marker::Array32.to_u8(), 0xff, 0xff, 0xff, 0xff],
            );
            assert_eq!(count, MAX_LEN + 5);
        }
    }
}
