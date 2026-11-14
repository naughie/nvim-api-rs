use super::{Primitive, Sealed, State, num};

impl Sealed for f32 {}
impl Primitive for f32 {
    type State = F32State;
    fn into_state(self) -> Self::State {
        F32State(Some(Repr(num::with_marker(self))))
    }
}
impl Sealed for &f32 {}
impl Primitive for &f32 {
    type State = <f32 as Primitive>::State;
    fn into_state(self) -> Self::State {
        Primitive::into_state(*self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct F32State(Option<Repr<5>>);

impl State for F32State {
    type Item = Repr<5>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.take()
    }
}

impl Sealed for f64 {}
impl Primitive for f64 {
    type State = F64State;
    fn into_state(self) -> Self::State {
        F64State(Some(Repr(num::with_marker(self))))
    }
}
impl Sealed for &f64 {}
impl Primitive for &f64 {
    type State = <f64 as Primitive>::State;
    fn into_state(self) -> Self::State {
        Primitive::into_state(*self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct F64State(Option<Repr<9>>);

impl State for F64State {
    type Item = Repr<9>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.take()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Repr<const N: usize>([u8; N]);
impl<const N: usize> AsRef<[u8]> for Repr<N> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::super::encode_to_vec;
    use rmp::Marker;

    #[test]
    fn eq_encoded() {
        let expected = {
            let mut buf = vec![Marker::F32.to_u8()];
            buf.extend_from_slice(&1.0f32.to_be_bytes());
            buf
        };
        assert_eq!(encode_to_vec(1.0f32), expected);
        let expected = {
            let mut buf = vec![Marker::F64.to_u8()];
            buf.extend_from_slice(&1.0f64.to_be_bytes());
            buf
        };
        assert_eq!(encode_to_vec(1.0f64), expected);
    }
}
