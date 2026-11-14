use super::{Primitive, Sealed};

/// Computes a primitive when its encoding state is created.
///
/// The closure is called by [`Primitive::into_state()`], before byte chunks are
/// written. Borrow `&Func<F>` or `&mut Func<F>` to use an `Fn` or `FnMut` again.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Func<F>(pub F);

impl<T: Primitive, F: FnOnce() -> T> Sealed for Func<F> {}
impl<T: Primitive, F: FnOnce() -> T> Primitive for Func<F> {
    type State = T::State;
    fn into_state(self) -> Self::State {
        self.0().into_state()
    }
}

impl<T: Primitive, F: FnMut() -> T> Sealed for &mut Func<F> {}
impl<T: Primitive, F: FnMut() -> T> Primitive for &mut Func<F> {
    type State = T::State;
    fn into_state(self) -> Self::State {
        self.0().into_state()
    }
}

impl<T: Primitive, F: Fn() -> T> Sealed for &Func<F> {}
impl<T: Primitive, F: Fn() -> T> Primitive for &Func<F> {
    type State = T::State;
    fn into_state(self) -> Self::State {
        self.0().into_state()
    }
}
