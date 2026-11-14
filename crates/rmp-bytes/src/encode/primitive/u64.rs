use super::{Primitive, Sealed, State, num};

impl Sealed for u64 {}
impl Primitive for u64 {
    type State = U64State;
    fn into_state(self) -> Self::State {
        match self {
            0..=0x7f => U64State(Some(Repr::Fixpos(num::fix_pos(self as u8)))),
            0x80..=0xff => U64State(Some(Repr::U8(num::with_marker(self as u8)))),
            0x100..=0xffff => U64State(Some(Repr::U16(num::with_marker(self as u16)))),
            0x10000..=0xffff_ffff => U64State(Some(Repr::U32(num::with_marker(self as u32)))),
            _ => U64State(Some(Repr::U64(num::with_marker(self)))),
        }
    }
}
impl Sealed for &u64 {}
impl Primitive for &u64 {
    type State = <u64 as Primitive>::State;
    fn into_state(self) -> Self::State {
        Primitive::into_state(*self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct U64State(Option<Repr>);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Repr {
    Fixpos([u8; 1]),
    U8([u8; 2]),
    U16([u8; 3]),
    U32([u8; 5]),
    U64([u8; 9]),
}
impl AsRef<[u8]> for Repr {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Fixpos(v) => v,
            Self::U8(v) => v,
            Self::U16(v) => v,
            Self::U32(v) => v,
            Self::U64(v) => v,
        }
    }
}

impl State for U64State {
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
        assert_eq!(encode_to_vec::<u64>(0), [Marker::FixPos(0).to_u8()]);
        assert_eq!(encode_to_vec::<u64>(0x7f), [Marker::FixPos(0x7f).to_u8()]);
        assert_eq!(encode_to_vec::<u64>(0x80), [Marker::U8.to_u8(), 0x80]);
        assert_eq!(encode_to_vec::<u64>(0xff), [Marker::U8.to_u8(), 0xff]);

        assert_eq!(
            encode_to_vec::<u64>(0x100),
            [Marker::U16.to_u8(), 0x01, 0x00]
        );
        assert_eq!(
            encode_to_vec::<u64>(0xffff),
            [Marker::U16.to_u8(), 0xff, 0xff]
        );

        assert_eq!(
            encode_to_vec::<u64>(0x10000),
            [Marker::U32.to_u8(), 0x00, 0x01, 0x00, 0x00]
        );
        assert_eq!(
            encode_to_vec::<u64>(0x1234_5678),
            [Marker::U32.to_u8(), 0x12, 0x34, 0x56, 0x78]
        );
        assert_eq!(
            encode_to_vec::<u64>(0xffff_ffff),
            [Marker::U32.to_u8(), 0xff, 0xff, 0xff, 0xff]
        );

        assert_eq!(
            encode_to_vec::<u64>(0x1_0000_0000),
            [
                Marker::U64.to_u8(),
                0x00,
                0x00,
                0x00,
                0x01,
                0x00,
                0x00,
                0x00,
                0x00
            ]
        );
        assert_eq!(
            encode_to_vec::<u64>(0x1234_5678_90ab_cdef),
            [
                Marker::U64.to_u8(),
                0x12,
                0x34,
                0x56,
                0x78,
                0x90,
                0xab,
                0xcd,
                0xef
            ]
        );
        assert_eq!(
            encode_to_vec::<u64>(0xffff_ffff_ffff_ffff),
            [
                Marker::U64.to_u8(),
                0xff,
                0xff,
                0xff,
                0xff,
                0xff,
                0xff,
                0xff,
                0xff
            ]
        );
    }
}
