use super::{Primitive, Sealed, State, num};

impl Sealed for u16 {}
impl Primitive for u16 {
    type State = U16State;
    fn into_state(self) -> Self::State {
        match self {
            0..=0x7f => U16State(Some(Repr::Fixpos(num::fix_pos(self as u8)))),
            0x80..=0xff => U16State(Some(Repr::U8(num::with_marker(self as u8)))),
            _ => U16State(Some(Repr::U16(num::with_marker(self)))),
        }
    }
}
impl Sealed for &u16 {}
impl Primitive for &u16 {
    type State = <u16 as Primitive>::State;
    fn into_state(self) -> Self::State {
        Primitive::into_state(*self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct U16State(Option<Repr>);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Repr {
    Fixpos([u8; 1]),
    U8([u8; 2]),
    U16([u8; 3]),
}
impl AsRef<[u8]> for Repr {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Fixpos(v) => v,
            Self::U8(v) => v,
            Self::U16(v) => v,
        }
    }
}

impl State for U16State {
    type Item = Repr;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.take()
    }
}

#[cfg(test)]
mod tests {
    use super::super::encode_to_vec;
    use rmp::Marker;

    #[test]
    fn eq_encoded() {
        assert_eq!(encode_to_vec::<u16>(0), [Marker::FixPos(0).to_u8()]);
        assert_eq!(encode_to_vec::<u16>(0x7f), [Marker::FixPos(0x7f).to_u8()]);
        assert_eq!(encode_to_vec::<u16>(0x80), [Marker::U8.to_u8(), 0x80]);
        assert_eq!(encode_to_vec::<u16>(0xff), [Marker::U8.to_u8(), 0xff]);

        assert_eq!(
            encode_to_vec::<u16>(0x100),
            [Marker::U16.to_u8(), 0x01, 0x00]
        );
        assert_eq!(
            encode_to_vec::<u16>(0xffff),
            [Marker::U16.to_u8(), 0xff, 0xff]
        );
    }
}
