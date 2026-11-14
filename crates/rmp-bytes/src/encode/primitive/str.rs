use super::MAX_LEN;
use super::StrLike;
use super::{Primitive, Sealed, State, data};
use rmp::Marker;

use std::num::NonZeroUsize;

impl Sealed for &str {}
impl<'a> Primitive for &'a str {
    type State = StrState<'a>;

    fn into_state(self) -> Self::State {
        StrState(self, StateInner::Marker)
    }
}
impl StrLike for &str {}

impl Sealed for &&str {}
impl<'s> Primitive for &&'s str {
    type State = <&'s str as Primitive>::State;

    fn into_state(self) -> Self::State {
        <&'s str as Primitive>::into_state(*self)
    }
}
impl StrLike for &&str {}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LenType {
    Fix([u8; 1]),
    U8([u8; 2]),
    U16([u8; 3]),
    U32([u8; 5]),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Repr<T> {
    Marker(LenType),
    Data(T),
}
impl<T: AsRef<[u8]>> AsRef<[u8]> for Repr<T> {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Marker(LenType::Fix(v)) => v,
            Self::Marker(LenType::U8(v)) => v,
            Self::Marker(LenType::U16(v)) => v,
            Self::Marker(LenType::U32(v)) => v,
            Self::Data(v) => v.as_ref(),
        }
    }
}

fn metadata(s: &str) -> (LenType, Option<NonZeroUsize>) {
    let len = s.len();
    match len {
        0..=0x1f => (LenType::Fix(data::marker(Marker::FixStr(len as u8))), None),
        0x20..=0xff => (LenType::U8(data::len(Marker::Str8, len as u8)), None),
        0x100..=0xffff => (LenType::U16(data::len(Marker::Str16, len as u16)), None),
        0x10000..=0xffff_ffff => (LenType::U32(data::len(Marker::Str32, len as u32)), None),
        _ => {
            let idx = s.floor_char_boundary(MAX_LEN);
            (
                LenType::U32(data::len(Marker::Str32, idx as u32)),
                NonZeroUsize::new(idx),
            )
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StateInner {
    Marker,
    Data,
    Done,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StrState<'a>(&'a str, StateInner);

impl<'a> State for StrState<'a> {
    type Item = Repr<&'a str>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.1 {
            StateInner::Marker => {
                self.1 = StateInner::Data;
                let (len, new_len) = metadata(self.0);
                if let Some(new_len) = new_len {
                    self.0 = &self.0[..new_len.get()];
                }

                Some(Repr::Marker(len))
            }
            StateInner::Data => {
                self.1 = StateInner::Done;
                Some(Repr::Data(self.0))
            }
            StateInner::Done => None,
        }
    }
}

impl Sealed for String {}
impl Primitive for String {
    type State = StringState;

    fn into_state(self) -> Self::State {
        StringState(Some(self), StateInner::Marker)
    }
}
impl StrLike for String {}

impl Sealed for &String {}
impl<'a> Primitive for &'a String {
    type State = <&'a str as Primitive>::State;
    fn into_state(self) -> Self::State {
        Primitive::into_state(&**self)
    }
}
impl StrLike for &String {}

#[derive(Clone, PartialEq, Eq)]
pub struct StringState(Option<String>, StateInner);

impl State for StringState {
    type Item = Repr<String>;

    fn next(&mut self) -> Option<Self::Item> {
        let inner = self.0.as_mut()?;
        match self.1 {
            StateInner::Marker => {
                self.1 = StateInner::Data;
                let (len, new_len) = metadata(inner);
                if let Some(new_len) = new_len {
                    inner.truncate(new_len.get());
                }

                Some(Repr::Marker(len))
            }
            StateInner::Data => {
                self.1 = StateInner::Done;
                self.0.take().map(Repr::Data)
            }
            StateInner::Done => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::encode_to_vec;
    use super::*;
    use rmp::Marker;

    #[test]
    fn owned_buffer_is_retained() {
        for (text, marker) in [
            ("", &[0xa0][..]),
            ("foo bar", &[0xa7]),
            ("non-ASCII: 漢字", &[0xb1]),
        ] {
            let mut s = String::with_capacity(text.len() + 16);
            s.push_str(text);
            let ptr = s.as_ptr();
            let capacity = s.capacity();
            let mut state = s.into_state();

            assert_eq!(state.next().unwrap().as_ref(), marker);
            let Some(Repr::Data(s)) = state.next() else {
                panic!("expected string data");
            };
            assert_eq!(s, text);
            assert_eq!(s.as_ptr(), ptr);
            assert_eq!(s.capacity(), capacity);
            assert!(state.next().is_none());
        }
    }

    #[test]
    fn eq_encoded() {
        fn assert_str<const N: usize>(s: &str, marker: [u8; N]) {
            let res = encode_to_vec::<&str>(s);

            assert!(res.starts_with(&marker));

            let expected = if s.len() <= MAX_LEN {
                s
            } else {
                let idx = s.floor_char_boundary(MAX_LEN);
                &s[..idx]
            };
            assert_eq!(&res[N..], expected.as_bytes());
        }
        fn assert_string<const N: usize>(s: String, marker: [u8; N]) {
            let res = encode_to_vec::<String>(s.clone());

            assert!(res.starts_with(&marker));

            let expected = if s.len() <= MAX_LEN {
                &s
            } else {
                let idx = s.floor_char_boundary(MAX_LEN);
                &s[..idx]
            };
            assert_eq!(&res[N..], expected.as_bytes());
        }

        assert_str("", [Marker::FixStr(0).to_u8()]);
        assert_str("foo bar", [Marker::FixStr(7).to_u8()]);
        assert_str("non-ASCII: 漢字", [Marker::FixStr(17).to_u8()]);

        {
            let long = "0123456789abcdef".repeat(2);
            assert_eq!(long.len(), 0x20);
            assert_str(&long, [Marker::Str8.to_u8(), 0x20]);
            assert_string(long, [Marker::Str8.to_u8(), 0x20]);
        }
        {
            let long = "0123456789abcdef".repeat(0x10);
            assert_eq!(long.len(), 0xff + 1);
            assert_str(&long, [Marker::Str16.to_u8(), 0x01, 0x00]);
            assert_string(long, [Marker::Str16.to_u8(), 0x01, 0x00]);
        }
        {
            let long = "0123456789abcdef".repeat(0x1000);
            assert_eq!(long.len(), 0xffff + 1);
            assert_str(&long, [Marker::Str32.to_u8(), 0x00, 0x01, 0x00, 0x00]);
            assert_string(long, [Marker::Str32.to_u8(), 0x00, 0x01, 0x00, 0x00]);
        }
    }
}
