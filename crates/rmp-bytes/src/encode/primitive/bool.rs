use super::{Primitive, Sealed, State};
use rmp::Marker;

impl Sealed for bool {}
impl Primitive for bool {
    type State = BoolState;
    fn into_state(self) -> Self::State {
        let marker = if self { Marker::True } else { Marker::False };
        BoolState(Some(Repr([marker.to_u8()])))
    }
}
impl Sealed for &bool {}
impl Primitive for &bool {
    type State = BoolState;
    fn into_state(self) -> Self::State {
        Primitive::into_state(*self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BoolState(Option<Repr>);
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Repr([u8; 1]);

impl AsRef<[u8]> for Repr {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl State for BoolState {
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
        assert_eq!(encode_to_vec(false), [Marker::False.to_u8()]);
        assert_eq!(encode_to_vec(true), [Marker::True.to_u8()]);
    }
}
