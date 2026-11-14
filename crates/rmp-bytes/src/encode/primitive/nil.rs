use super::{Primitive, Sealed, State};
use rmp::Marker;

/// MessagePack nil (`0xc0`). Use `()` for an empty array instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Nil;

impl Sealed for Nil {}
impl Primitive for Nil {
    type State = NilState;
    fn into_state(self) -> Self::State {
        NilState(Some(Repr([Marker::Null.to_u8()])))
    }
}
impl Sealed for &Nil {}
impl Primitive for &Nil {
    type State = <Nil as Primitive>::State;
    fn into_state(self) -> Self::State {
        Primitive::into_state(*self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct NilState(Option<Repr>);
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Repr([u8; 1]);

impl AsRef<[u8]> for Repr {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl State for NilState {
    type Item = Repr;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.take()
    }
}

#[cfg(test)]
mod tests {
    use super::super::encode_to_vec;
    use super::*;

    #[test]
    fn eq_encoded() {
        assert_eq!(encode_to_vec(Nil), [Marker::Null.to_u8()]);
    }
}
