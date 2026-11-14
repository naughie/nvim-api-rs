use super::{Primitive, Sealed, State};

use std::convert::Infallible;

impl Sealed for Infallible {}
impl Primitive for Infallible {
    type State = InfallibleState;
    fn into_state(self) -> Self::State {
        match self {}
    }
}

pub enum InfallibleState {}

impl AsRef<[u8]> for InfallibleState {
    fn as_ref(&self) -> &[u8] {
        match *self {}
    }
}

impl State for InfallibleState {
    type Item = InfallibleState;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
