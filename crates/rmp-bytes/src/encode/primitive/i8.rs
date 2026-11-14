use super::{Primitive, Sealed, State, num};

impl Sealed for i8 {}
impl Primitive for i8 {
    type State = I8State;
    fn into_state(self) -> Self::State {
        match self {
            0..=0x7f => I8State(Some(Repr::Fixpos(num::fix_pos(self as u8)))),
            -0x20..0 => I8State(Some(Repr::Fixneg(num::fix_neg(self)))),
            _ => I8State(Some(Repr::I8(num::with_marker(self)))),
        }
    }
}
impl Sealed for &i8 {}
impl Primitive for &i8 {
    type State = <i8 as Primitive>::State;
    fn into_state(self) -> Self::State {
        Primitive::into_state(*self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct I8State(Option<Repr>);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Repr {
    Fixpos([u8; 1]),
    Fixneg([u8; 1]),
    I8([u8; 2]),
}
impl AsRef<[u8]> for Repr {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Fixpos(v) => v,
            Self::Fixneg(v) => v,
            Self::I8(v) => v,
        }
    }
}

impl State for I8State {
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
        assert_eq!(encode_to_vec::<i8>(0), [Marker::FixPos(0).to_u8()]);
        assert_eq!(encode_to_vec::<i8>(0x7f), [Marker::FixPos(0x7f).to_u8()]);
        assert_eq!(encode_to_vec::<i8>(-1), [Marker::FixNeg(-1).to_u8()]);
        assert_eq!(encode_to_vec::<i8>(-32), [Marker::FixNeg(-32).to_u8()]);
        assert_eq!(encode_to_vec::<i8>(-33), [Marker::I8.to_u8(), 0xdf]);
        assert_eq!(encode_to_vec::<i8>(-0x80), [Marker::I8.to_u8(), 0x80]);
    }
}
