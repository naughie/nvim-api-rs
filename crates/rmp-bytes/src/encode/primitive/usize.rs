use super::{Primitive, Sealed};

impl Sealed for usize {}

#[cfg(target_pointer_width = "16")]
impl Primitive for usize {
    type State = <u16 as Primitive>::State;

    fn into_state(self) -> Self::State {
        (self as u16).into_state()
    }
}
#[cfg(target_pointer_width = "32")]
impl Primitive for usize {
    type State = <u32 as Primitive>::State;

    fn into_state(self) -> Self::State {
        (self as u32).into_state()
    }
}
#[cfg(target_pointer_width = "64")]
impl Primitive for usize {
    type State = <u64 as Primitive>::State;

    fn into_state(self) -> Self::State {
        (self as u64).into_state()
    }
}

impl Sealed for &usize {}
#[cfg(any(
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64"
))]
impl Primitive for &usize {
    type State = <usize as Primitive>::State;
    fn into_state(self) -> Self::State {
        Primitive::into_state(*self)
    }
}
