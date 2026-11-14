use super::{MAX_LEN, MAX_LEN_U32};
use super::{Primitive, Sealed, State, data};
use rmp::Marker;

impl Sealed for &[u8] {}
impl<'a> Primitive for &'a [u8] {
    type State = ByteState<&'a [u8]>;

    fn into_state(self) -> Self::State {
        let len = metadata(self.len());
        let bytes = if self.len() > MAX_LEN {
            &self[..MAX_LEN]
        } else {
            self
        };
        ByteState(Some(len), Some(bytes))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LenType {
    U8([u8; 2]),
    U16([u8; 3]),
    U32([u8; 5]),
}

fn metadata(len: usize) -> LenType {
    match len {
        0..=0xff => LenType::U8(data::len(Marker::Bin8, len as u8)),
        0x100..=0xffff => LenType::U16(data::len(Marker::Bin16, len as u16)),
        0x10000..=0xffff_ffff => LenType::U32(data::len(Marker::Bin32, len as u32)),
        _ => LenType::U32(data::len(Marker::Bin32, MAX_LEN_U32)),
    }
}

impl Sealed for Vec<u8> {}
impl Primitive for Vec<u8> {
    type State = ByteState<Vec<u8>>;

    fn into_state(mut self) -> Self::State {
        let len = metadata(self.len());
        if self.len() > MAX_LEN {
            // SAFETY: MAX_LEN <= self.len() <= capacity
            unsafe {
                self.set_len(MAX_LEN);
            }
        }
        ByteState(Some(len), Some(self))
    }
}
impl Sealed for &Vec<u8> {}
impl<'a> Primitive for &'a Vec<u8> {
    type State = <&'a [u8] as Primitive>::State;
    fn into_state(self) -> Self::State {
        Primitive::into_state(&**self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ByteState<T>(Option<LenType>, Option<T>);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Repr<T> {
    Marker(LenType),
    Data(T),
}

impl<T: AsRef<[u8]>> AsRef<[u8]> for Repr<T> {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Marker(LenType::U8(v)) => v,
            Self::Marker(LenType::U16(v)) => v,
            Self::Marker(LenType::U32(v)) => v,
            Self::Data(v) => v.as_ref(),
        }
    }
}

impl<T: AsRef<[u8]>> State for ByteState<T> {
    type Item = Repr<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(len) = self.0.take() {
            return Some(Repr::Marker(len));
        }
        self.1.take().map(Repr::Data)
    }
}

/// Encodes an exactly sized iterator of bytes as a MessagePack binary value.
///
/// To encode an array of integer values instead, use [`super::ArrayIter`].
pub struct BinIter<I>(pub I);

impl<I: ExactSizeIterator<Item = u8>> Sealed for BinIter<I> {}
impl<I: ExactSizeIterator<Item = u8>> Primitive for BinIter<I> {
    type State = BinIterState<I>;

    fn into_state(self) -> Self::State {
        let len = metadata(self.0.len());
        BinIterState(self.0, Some(len), [0], 0)
    }
}

pub struct BinIterState<I>(I, Option<LenType>, [u8; 1], u32);

impl<I: ExactSizeIterator<Item = u8>> State for BinIterState<I> {
    type Item = Repr<[u8; 1]>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(len) = self.1.take() {
            return Some(Repr::Marker(len));
        }
        if self.3 == MAX_LEN_U32 {
            return None;
        }
        let next = self.0.next()?;
        self.3 += 1;
        self.2[0] = next;
        Some(Repr::Data(self.2))
    }
}

#[cfg(test)]
mod tests {
    use super::super::count_encoded_with_marker;
    use super::super::encode_to_vec;
    use super::*;
    use rmp::Marker;

    #[test]
    fn eq_encoded() {
        fn assert_slice<const N: usize>(s: &[u8], marker: [u8; N]) {
            let res = encode_to_vec::<&[u8]>(s);
            assert!(res.starts_with(&marker));
            assert_eq!(&res[N..], s);
        }
        fn assert_vec<const N: usize>(s: Vec<u8>, marker: [u8; N]) {
            let res = encode_to_vec::<Vec<u8>>(s.clone());
            assert!(res.starts_with(&marker));
            assert_eq!(&res[N..], s);
        }

        assert_slice(b"", [Marker::Bin8.to_u8(), 0]);
        assert_slice(b"foo bar", [Marker::Bin8.to_u8(), 7]);
        assert_vec(b"foo bar".to_vec(), [Marker::Bin8.to_u8(), 7]);

        {
            let count = count_encoded_with_marker(
                BinIter((0..0x100).map(|_| b'a')),
                &[Marker::Bin16.to_u8(), 0x01, 0x00],
            );
            assert_eq!(count, 0x100 + 3);
        }
        {
            let count = count_encoded_with_marker(
                BinIter((0..0x10000).map(|_| b'a')),
                &[Marker::Bin32.to_u8(), 0x00, 0x01, 0x00, 0x00],
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
                BinIter((0..MAX_LEN).map(|_| b'a')),
                &[Marker::Bin32.to_u8(), 0xff, 0xff, 0xff, 0xff],
            );
            assert_eq!(count, MAX_LEN + 5);
        }
        {
            let count = count_encoded_with_marker(
                BinIter((0..(MAX_LEN + 1)).map(|_| b'a')),
                &[Marker::Bin32.to_u8(), 0xff, 0xff, 0xff, 0xff],
            );
            assert_eq!(count, MAX_LEN + 5);
        }
    }
}
