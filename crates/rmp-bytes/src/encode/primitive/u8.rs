use super::{Primitive, Sealed, State, num};

impl Sealed for u8 {}
impl Primitive for u8 {
    type State = U8State;
    fn into_state(self) -> Self::State {
        if self <= 0x7f {
            U8State(Some(Repr::Fixpos(num::fix_pos(self))))
        } else {
            U8State(Some(Repr::U8(num::with_marker(self))))
        }
    }
}
impl Sealed for &u8 {}
impl Primitive for &u8 {
    type State = <u8 as Primitive>::State;
    fn into_state(self) -> Self::State {
        Primitive::into_state(*self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct U8State(Option<Repr>);
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Repr {
    Fixpos([u8; 1]),
    U8([u8; 2]),
}

impl AsRef<[u8]> for Repr {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Fixpos(v) => v,
            Self::U8(v) => v,
        }
    }
}

impl State for U8State {
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
        assert_eq!(encode_to_vec::<u8>(0), [Marker::FixPos(0).to_u8()]);
        assert_eq!(encode_to_vec::<u8>(0x7f), [Marker::FixPos(0x7f).to_u8()]);
        assert_eq!(encode_to_vec::<u8>(0x80), [Marker::U8.to_u8(), 0x80]);
        assert_eq!(encode_to_vec::<u8>(0xff), [Marker::U8.to_u8(), 0xff]);
    }
}
