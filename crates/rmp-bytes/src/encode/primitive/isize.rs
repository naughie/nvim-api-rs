use super::{Primitive, Sealed};

impl Sealed for isize {}

#[cfg(target_pointer_width = "16")]
impl Primitive for isize {
    type State = <i16 as Primitive>::State;

    fn into_state(self) -> Self::State {
        (self as i16).into_state()
    }
}
#[cfg(target_pointer_width = "32")]
impl Primitive for isize {
    type State = <i32 as Primitive>::State;

    fn into_state(self) -> Self::State {
        (self as i32).into_state()
    }
}
#[cfg(target_pointer_width = "64")]
impl Primitive for isize {
    type State = <i64 as Primitive>::State;

    fn into_state(self) -> Self::State {
        (self as i64).into_state()
    }
}

impl Sealed for &isize {}
#[cfg(any(
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64"
))]
impl Primitive for &isize {
    type State = <isize as Primitive>::State;
    fn into_state(self) -> Self::State {
        Primitive::into_state(*self)
    }
}
