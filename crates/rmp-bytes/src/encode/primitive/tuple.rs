use super::ArrayLike;
use super::{Primitive, Sealed, State};
use rmp::Marker;

impl Sealed for () {}
impl Primitive for () {
    type State = VoidState;
    fn into_state(self) -> Self::State {
        VoidState(Some([Marker::FixArray(0).to_u8()]))
    }
}
impl ArrayLike for () {}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VoidState(Option<[u8; 1]>);

impl State for VoidState {
    type Item = [u8; 1];
    fn next(&mut self) -> Option<Self::Item> {
        self.0.take()
    }
}

macro_rules! _impl_prim_for_tuple {
    ($num:expr, $priv:ident, $first:ident, $( ( $ty:ident => $next_ty:ident ),)* ) => {
        impl<$( $ty: Primitive, )*> Sealed for ($( $ty, )*) {}
        impl<$( $ty: Primitive, )*> Primitive for ($( $ty, )*) {
            type State = $priv::TupleState<$( $ty, )*>;

            fn into_state(self) -> Self::State {
                #[allow(non_snake_case)]
                let ( $( $ty, )* ) = self;
                $priv::TupleState {
                    which: $priv::Which::Len,
                    $( $ty: Some( $ty ), )*
                }
            }
        }
        impl<$( $ty: Primitive, )*> ArrayLike for ($( $ty, )*) {}

        mod $priv {
            use super::{State, Primitive};
            use rmp::Marker;

            #[derive(Clone, Copy, PartialEq, Eq)]
            pub(super) enum Which<$( $ty, )*> {
                Len,
                $( $ty (Option< $ty >), )*
                Done,
            }

            impl<$( $ty, )*> Which<$( $ty, )*> {
                fn done(_opt: Option<()>) -> Self {
                    Self::Done
                }
            }

            #[derive(Clone, Copy, PartialEq, Eq)]
            pub enum Item<$($ty: AsRef<[u8]>, )*> {
                Len([u8; 1]),
                $( $ty ($ty), )*
            }
            impl<$( $ty: AsRef<[u8]>, )*> AsRef<[u8]> for Item<$( $ty, )*> {
                fn as_ref(&self) -> &[u8] {
                    match self {
                        Self::Len(v) => v,
                        $( Self::$ty (v) => v.as_ref(), )*
                    }
                }
            }

            #[allow(non_snake_case)]
            pub struct TupleState<$( $ty: Primitive, )*> {
                pub(super) which: Which<$( <$ty as Primitive>::State, )*>,
                $( pub(super) $ty: Option<$ty>, )*
            }

            impl<$( $ty: Primitive, )*> State for TupleState<$( $ty, )*> {
                type Item = Item<$( <<$ty as Primitive>::State as State>::Item, )*>;

                fn next(&mut self) -> Option<Self::Item> {
                    loop {
                        match &mut self.which {
                            Which::Len => {
                                self.which = Which::$first (None);
                                return Some(Item::Len([Marker::FixArray($num).to_u8()]))
                            }
                            $(
                                Which::$ty (state) => {
                                    let state = state.get_or_insert_with(|| self.$ty.take().expect("should be called once").into_state());
                                    let next = state.next();

                                    if let Some(next) = next {
                                        return Some(Item::$ty (next));
                                    } else {
                                        self.which = Which::$next_ty (None);
                                    }
                                }
                            )*
                            Which::Done => return None,
                        }
                    }
                }
            }
        }
    };
}

_impl_prim_for_tuple! {
    1,
    _tuple1,
    T1,
    ( T1 => done ),
}
_impl_prim_for_tuple! {
    2,
    _tuple2,
    T1,
    ( T1 => T2 ),
    ( T2 => done ),
}
_impl_prim_for_tuple! {
    3,
    _tuple3,
    T1,
    ( T1 => T2 ),
    ( T2 => T3 ),
    ( T3 => done ),
}
_impl_prim_for_tuple! {
    4,
    _tuple4,
    T1,
    ( T1 => T2 ),
    ( T2 => T3 ),
    ( T3 => T4 ),
    ( T4 => done ),
}
_impl_prim_for_tuple! {
    5,
    _tuple5,
    T1,
    ( T1 => T2 ),
    ( T2 => T3 ),
    ( T3 => T4 ),
    ( T4 => T5 ),
    ( T5 => done ),
}
_impl_prim_for_tuple! {
    6,
    _tuple6,
    T1,
    ( T1 => T2 ),
    ( T2 => T3 ),
    ( T3 => T4 ),
    ( T4 => T5 ),
    ( T5 => T6 ),
    ( T6 => done ),
}
_impl_prim_for_tuple! {
    7,
    _tuple7,
    T1,
    ( T1 => T2 ),
    ( T2 => T3 ),
    ( T3 => T4 ),
    ( T4 => T5 ),
    ( T5 => T6 ),
    ( T6 => T7 ),
    ( T7 => done ),
}
_impl_prim_for_tuple! {
    8,
    _tuple8,
    T1,
    ( T1 => T2 ),
    ( T2 => T3 ),
    ( T3 => T4 ),
    ( T4 => T5 ),
    ( T5 => T6 ),
    ( T6 => T7 ),
    ( T7 => T8 ),
    ( T8 => done ),
}
_impl_prim_for_tuple! {
    9,
    _tuple9,
    T1,
    ( T1 => T2 ),
    ( T2 => T3 ),
    ( T3 => T4 ),
    ( T4 => T5 ),
    ( T5 => T6 ),
    ( T6 => T7 ),
    ( T7 => T8 ),
    ( T8 => T9 ),
    ( T9 => done ),
}
_impl_prim_for_tuple! {
    10,
    _tuple10,
    T1,
    ( T1 => T2 ),
    ( T2 => T3 ),
    ( T3 => T4 ),
    ( T4 => T5 ),
    ( T5 => T6 ),
    ( T6 => T7 ),
    ( T7 => T8 ),
    ( T8 => T9 ),
    ( T9 => T10 ),
    ( T10 => done ),
}
_impl_prim_for_tuple! {
    11,
    _tuple11,
    T1,
    ( T1 => T2 ),
    ( T2 => T3 ),
    ( T3 => T4 ),
    ( T4 => T5 ),
    ( T5 => T6 ),
    ( T6 => T7 ),
    ( T7 => T8 ),
    ( T8 => T9 ),
    ( T9 => T10 ),
    ( T10 => T11 ),
    ( T11 => done ),
}
_impl_prim_for_tuple! {
    12,
    _tuple12,
    T1,
    ( T1 => T2 ),
    ( T2 => T3 ),
    ( T3 => T4 ),
    ( T4 => T5 ),
    ( T5 => T6 ),
    ( T6 => T7 ),
    ( T7 => T8 ),
    ( T8 => T9 ),
    ( T9 => T10 ),
    ( T10 => T11 ),
    ( T11 => T12 ),
    ( T12 => done ),
}
_impl_prim_for_tuple! {
    13,
    _tuple13,
    T1,
    ( T1 => T2 ),
    ( T2 => T3 ),
    ( T3 => T4 ),
    ( T4 => T5 ),
    ( T5 => T6 ),
    ( T6 => T7 ),
    ( T7 => T8 ),
    ( T8 => T9 ),
    ( T9 => T10 ),
    ( T10 => T11 ),
    ( T11 => T12 ),
    ( T12 => T13 ),
    ( T13 => done ),
}
_impl_prim_for_tuple! {
    14,
    _tuple14,
    T1,
    ( T1 => T2 ),
    ( T2 => T3 ),
    ( T3 => T4 ),
    ( T4 => T5 ),
    ( T5 => T6 ),
    ( T6 => T7 ),
    ( T7 => T8 ),
    ( T8 => T9 ),
    ( T9 => T10 ),
    ( T10 => T11 ),
    ( T11 => T12 ),
    ( T12 => T13 ),
    ( T13 => T14 ),
    ( T14 => done ),
}
_impl_prim_for_tuple! {
    15,
    _tuple15,
    T1,
    ( T1 => T2 ),
    ( T2 => T3 ),
    ( T3 => T4 ),
    ( T4 => T5 ),
    ( T5 => T6 ),
    ( T6 => T7 ),
    ( T7 => T8 ),
    ( T8 => T9 ),
    ( T9 => T10 ),
    ( T10 => T11 ),
    ( T11 => T12 ),
    ( T12 => T13 ),
    ( T13 => T14 ),
    ( T14 => T15 ),
    ( T15 => done ),
}

#[cfg(test)]
mod tests {
    use super::super::Array;
    use super::super::encode_to_vec;
    use rmp::Marker;

    #[test]
    fn eq_encoded() {
        assert_eq!(encode_to_vec(()), [Marker::FixArray(0).to_u8()]);

        assert_eq!(
            encode_to_vec((1u8,)),
            [Marker::FixArray(1).to_u8(), Marker::FixPos(1).to_u8()]
        );
        assert_eq!(
            encode_to_vec((1u8, "foo")),
            [
                Marker::FixArray(2).to_u8(),
                Marker::FixPos(1).to_u8(),
                Marker::FixStr(3).to_u8(),
                b'f',
                b'o',
                b'o',
            ]
        );

        let array = Array([2u8, 3u8]);

        assert_eq!(
            encode_to_vec((array, 1u8, "foo")),
            [
                Marker::FixArray(3).to_u8(),
                Marker::FixArray(2).to_u8(),
                Marker::FixPos(2).to_u8(),
                Marker::FixPos(3).to_u8(),
                Marker::FixPos(1).to_u8(),
                Marker::FixStr(3).to_u8(),
                b'f',
                b'o',
                b'o',
            ]
        );
        assert_eq!(
            encode_to_vec((1u8, array, "foo")),
            [
                Marker::FixArray(3).to_u8(),
                Marker::FixPos(1).to_u8(),
                Marker::FixArray(2).to_u8(),
                Marker::FixPos(2).to_u8(),
                Marker::FixPos(3).to_u8(),
                Marker::FixStr(3).to_u8(),
                b'f',
                b'o',
                b'o',
            ]
        );
        assert_eq!(
            encode_to_vec((1u8, "foo", array)),
            [
                Marker::FixArray(3).to_u8(),
                Marker::FixPos(1).to_u8(),
                Marker::FixStr(3).to_u8(),
                b'f',
                b'o',
                b'o',
                Marker::FixArray(2).to_u8(),
                Marker::FixPos(2).to_u8(),
                Marker::FixPos(3).to_u8(),
            ]
        );
    }
}
