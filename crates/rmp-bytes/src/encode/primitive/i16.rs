use super::{Primitive, Sealed, State, num};

impl Sealed for i16 {}
impl Primitive for i16 {
    type State = I16State;
    fn into_state(self) -> Self::State {
        match self {
            0..=0x7f => I16State(Some(Repr::Fixpos(num::fix_pos(self as u8)))),
            -0x20..0 => I16State(Some(Repr::Fixneg(num::fix_neg(self as i8)))),
            -0x80..-0x20 => I16State(Some(Repr::I8(num::with_marker(self as i8)))),
            0x80..=0xff => I16State(Some(Repr::U8(num::with_marker(self as u8)))),
            _ => I16State(Some(Repr::I16(num::with_marker(self)))),
        }
    }
}
impl Sealed for &i16 {}
impl Primitive for &i16 {
    type State = <i16 as Primitive>::State;
    fn into_state(self) -> Self::State {
        Primitive::into_state(*self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct I16State(Option<Repr>);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Repr {
    Fixpos([u8; 1]),
    Fixneg([u8; 1]),
    I8([u8; 2]),
    U8([u8; 2]),
    I16([u8; 3]),
}
impl AsRef<[u8]> for Repr {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Fixpos(v) => v,
            Self::Fixneg(v) => v,
            Self::I8(v) => v,
            Self::U8(v) => v,
            Self::I16(v) => v,
        }
    }
}

impl State for I16State {
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
        assert_eq!(encode_to_vec::<i16>(0), [Marker::FixPos(0).to_u8()]);
        assert_eq!(encode_to_vec::<i16>(0x7f), [Marker::FixPos(0x7f).to_u8()]);
        assert_eq!(encode_to_vec::<i16>(-1), [Marker::FixNeg(-1).to_u8()]);
        assert_eq!(encode_to_vec::<i16>(-32), [Marker::FixNeg(-32).to_u8()]);
        assert_eq!(encode_to_vec::<i16>(-33), [Marker::I8.to_u8(), 0xdf]);
        assert_eq!(encode_to_vec::<i16>(-0x80), [Marker::I8.to_u8(), 0x80]);

        assert_eq!(encode_to_vec::<i16>(0x80), [Marker::U8.to_u8(), 0x80]);
        assert_eq!(encode_to_vec::<i16>(0xff), [Marker::U8.to_u8(), 0xff]);
        assert_eq!(
            encode_to_vec::<i16>(0x100),
            [Marker::I16.to_u8(), 0x01, 0x00]
        );
        assert_eq!(
            encode_to_vec::<i16>(0x7fff),
            [Marker::I16.to_u8(), 0x7f, 0xff]
        );
        assert_eq!(
            encode_to_vec::<i16>(-0x81),
            [Marker::I16.to_u8(), 0xff, 0x7f]
        );
        assert_eq!(
            encode_to_vec::<i16>(-0x8000),
            [Marker::I16.to_u8(), 0x80, 0x00]
        );
    }
}
