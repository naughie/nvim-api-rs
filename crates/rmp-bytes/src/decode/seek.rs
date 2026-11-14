use super::{Buf, InsufficientBytes};

use rmp::Marker;

use std::num::NonZeroUsize;
use std::ops::ControlFlow;

/// Consumes one complete value and returns its byte length.
///
/// On [`InsufficientBytes`], the available prefix has been consumed. For input
/// arriving in chunks, use [`State`] to retain progress between calls.
///
/// # Examples
///
/// ```
/// use rmp_bytes::decode::{seek_to_eoo, InsufficientBytes};
/// use rmp_bytes::rmp::Marker;
///
/// use std::num::NonZeroUsize;
///
/// let mut bytes = &b""[..];
/// assert_eq!(seek_to_eoo(&mut bytes), Err(InsufficientBytes));
///
/// let mut bytes = &[Marker::Str8.to_u8(), 3, b'A', b'B', b'C'][..];
/// assert_eq!(seek_to_eoo(&mut bytes), Ok(NonZeroUsize::new(5).unwrap()));
/// assert_eq!(bytes, b"");
/// assert_eq!(seek_to_eoo(&mut bytes), Err(InsufficientBytes));
///
/// let mut bytes = &[Marker::Str8.to_u8(), 3, b'A', b'B', b'C', Marker::Null.to_u8(),
/// Marker::Array16.to_u8()][..];
/// assert_eq!(seek_to_eoo(&mut bytes), Ok(NonZeroUsize::new(5).unwrap()));
/// assert_eq!(bytes, [Marker::Null.to_u8(), Marker::Array16.to_u8()]);
/// assert_eq!(seek_to_eoo(&mut bytes), Ok(NonZeroUsize::new(1).unwrap()));
/// assert_eq!(bytes, [Marker::Array16.to_u8()]);
/// assert_eq!(seek_to_eoo(&mut bytes), Err(InsufficientBytes));
/// assert_eq!(bytes, b"");
/// ```
pub fn seek_to_eoo(cursor: &mut impl Buf) -> Result<NonZeroUsize, InsufficientBytes> {
    let mut state = State::new();
    if let ControlFlow::Break(len) = state.transit(cursor) {
        Ok(len)
    } else {
        Err(InsufficientBytes)
    }
}

/// Tracks one MessagePack value across successive input chunks.
///
/// Each call consumes the available prefix. `Continue(())` means more input is
/// needed. `Break(n)` reports bytes consumed in that call, not the total frame
/// length, and resets the state for the next value.
///
/// ```
/// use rmp_bytes::decode::SeekState;
/// use std::ops::ControlFlow;
///
/// let mut state = SeekState::new();
/// let mut first = &b"\xa3a"[..];
/// assert!(matches!(state.transit(&mut first), ControlFlow::Continue(())));
/// assert!(first.is_empty());
/// let mut second = &b"bc\xc0"[..];
/// let ControlFlow::Break(n) = state.transit(&mut second) else { panic!("incomplete") };
/// assert_eq!(n.get(), 2);
/// assert_eq!(second, b"\xc0");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    remaining_items: u64,
    reading: Reading,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub fn new() -> Self {
        Self {
            remaining_items: 1,
            reading: Reading::Marker,
        }
    }

    pub fn transit(&mut self, cursor: &mut impl Buf) -> ControlFlow<NonZeroUsize> {
        let mut total_read = 0usize;

        while self.remaining_items != 0 {
            if cursor.len() == 0 {
                return ControlFlow::Continue(());
            }

            let len = transit_impl(self, cursor);
            total_read += len;
        }

        if let Some(len) = NonZeroUsize::new(total_read) {
            *self = Self::new();
            ControlFlow::Break(len)
        } else {
            ControlFlow::Continue(())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reading {
    Marker,
    CompleteBytes { len: u64 },
    ByteLength { len: Length },
    ArrayLength { len: Length },
    MapLength { len: Length },
    ExtLength { len: Length },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Length {
    buf: [u8; 4],
    remaining: LengthToRead,
}

impl Length {
    fn new(len: LengthToRead) -> Self {
        Self {
            buf: [0u8; 4],
            remaining: len,
        }
    }

    fn to_u32(self) -> u32 {
        u32::from_be_bytes(self.buf)
    }

    fn update(mut self, cursor: &mut impl Buf) -> (Self, bool, usize) {
        match self.remaining {
            LengthToRead::One => {
                if let Some(byte) = cursor.first_byte() {
                    self.buf[3] = byte;
                    (self, true, 1)
                } else {
                    (self, false, 0)
                }
            }
            LengthToRead::Two => {
                let Some(byte) = cursor.first_byte() else {
                    return (self, false, 0);
                };
                self.buf[2] = byte;
                self.remaining = LengthToRead::One;
                let Some(byte) = cursor.first_byte() else {
                    return (self, false, 1);
                };
                self.buf[3] = byte;
                (self, true, 2)
            }
            LengthToRead::Three => {
                let Some(byte) = cursor.first_byte() else {
                    return (self, false, 0);
                };
                self.buf[1] = byte;
                self.remaining = LengthToRead::Two;
                let Some(byte) = cursor.first_byte() else {
                    return (self, false, 1);
                };
                self.buf[2] = byte;
                self.remaining = LengthToRead::One;
                let Some(byte) = cursor.first_byte() else {
                    return (self, false, 2);
                };
                self.buf[3] = byte;
                (self, true, 3)
            }
            LengthToRead::Four => {
                let Some(byte) = cursor.first_byte() else {
                    return (self, false, 0);
                };
                self.buf[0] = byte;
                self.remaining = LengthToRead::Three;
                let Some(byte) = cursor.first_byte() else {
                    return (self, false, 1);
                };
                self.buf[1] = byte;
                self.remaining = LengthToRead::Two;
                let Some(byte) = cursor.first_byte() else {
                    return (self, false, 2);
                };
                self.buf[2] = byte;
                self.remaining = LengthToRead::One;
                let Some(byte) = cursor.first_byte() else {
                    return (self, false, 3);
                };
                self.buf[3] = byte;
                (self, true, 4)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LengthToRead {
    One,
    Two,
    Three,
    Four,
}

fn transit_impl(state: &mut State, cursor: &mut impl Buf) -> usize {
    debug_assert_ne!(state.remaining_items, 0);

    match state.reading {
        Reading::Marker => {
            let Some(marker) = cursor.first_byte() else {
                return 0;
            };
            let marker = Marker::from_u8(marker);

            match marker {
                Marker::Null
                | Marker::True
                | Marker::False
                | Marker::FixPos(_)
                | Marker::FixNeg(_)
                | Marker::Reserved => state.remaining_items -= 1,

                Marker::U8 => state.reading = Reading::CompleteBytes { len: 1 },
                Marker::U16 => state.reading = Reading::CompleteBytes { len: 2 },
                Marker::U32 => state.reading = Reading::CompleteBytes { len: 4 },
                Marker::U64 => state.reading = Reading::CompleteBytes { len: 8 },

                Marker::I8 => state.reading = Reading::CompleteBytes { len: 1 },
                Marker::I16 => state.reading = Reading::CompleteBytes { len: 2 },
                Marker::I32 => state.reading = Reading::CompleteBytes { len: 4 },
                Marker::I64 => state.reading = Reading::CompleteBytes { len: 8 },

                Marker::F32 => state.reading = Reading::CompleteBytes { len: 4 },
                Marker::F64 => state.reading = Reading::CompleteBytes { len: 8 },

                Marker::FixStr(0) => state.remaining_items -= 1,
                Marker::FixStr(len) => state.reading = Reading::CompleteBytes { len: len as u64 },
                Marker::Str8 => {
                    state.reading = Reading::ByteLength {
                        len: Length::new(LengthToRead::One),
                    }
                }
                Marker::Str16 => {
                    state.reading = Reading::ByteLength {
                        len: Length::new(LengthToRead::Two),
                    }
                }
                Marker::Str32 => {
                    state.reading = Reading::ByteLength {
                        len: Length::new(LengthToRead::Four),
                    }
                }

                Marker::Bin8 => {
                    state.reading = Reading::ByteLength {
                        len: Length::new(LengthToRead::One),
                    }
                }
                Marker::Bin16 => {
                    state.reading = Reading::ByteLength {
                        len: Length::new(LengthToRead::Two),
                    }
                }
                Marker::Bin32 => {
                    state.reading = Reading::ByteLength {
                        len: Length::new(LengthToRead::Four),
                    }
                }

                Marker::FixArray(len) => {
                    state.remaining_items = state.remaining_items - 1 + len as u64
                }
                Marker::Array16 => {
                    state.reading = Reading::ArrayLength {
                        len: Length::new(LengthToRead::Two),
                    }
                }
                Marker::Array32 => {
                    state.reading = Reading::ArrayLength {
                        len: Length::new(LengthToRead::Four),
                    }
                }

                Marker::FixMap(len) => {
                    state.remaining_items = state.remaining_items - 1 + 2 * len as u64
                }
                Marker::Map16 => {
                    state.reading = Reading::MapLength {
                        len: Length::new(LengthToRead::Two),
                    }
                }
                Marker::Map32 => {
                    state.reading = Reading::MapLength {
                        len: Length::new(LengthToRead::Four),
                    }
                }

                Marker::FixExt1 => state.reading = Reading::CompleteBytes { len: 2 },
                Marker::FixExt2 => state.reading = Reading::CompleteBytes { len: 3 },
                Marker::FixExt4 => state.reading = Reading::CompleteBytes { len: 5 },
                Marker::FixExt8 => state.reading = Reading::CompleteBytes { len: 9 },
                Marker::FixExt16 => state.reading = Reading::CompleteBytes { len: 17 },

                Marker::Ext8 => {
                    state.reading = Reading::ExtLength {
                        len: Length::new(LengthToRead::One),
                    }
                }
                Marker::Ext16 => {
                    state.reading = Reading::ExtLength {
                        len: Length::new(LengthToRead::Two),
                    }
                }
                Marker::Ext32 => {
                    state.reading = Reading::ExtLength {
                        len: Length::new(LengthToRead::Four),
                    }
                }
            }

            1
        }
        Reading::CompleteBytes { len } => {
            if len <= cursor.len() as u64 {
                let len_usize = len as usize;
                cursor.advance(len_usize);
                state.remaining_items -= 1;
                state.reading = Reading::Marker;
                len_usize
            } else {
                let cursor_len = cursor.len();
                cursor.advance(cursor_len);
                state.reading = Reading::CompleteBytes {
                    len: len - cursor_len as u64,
                };
                cursor_len
            }
        }
        Reading::ByteLength { len } => {
            let (len, is_full, read) = len.update(cursor);
            if is_full {
                let len = u64::from(len.to_u32());
                if len == 0 {
                    state.remaining_items -= 1;
                    state.reading = Reading::Marker;
                } else {
                    state.reading = Reading::CompleteBytes { len };
                }
            } else {
                state.reading = Reading::ByteLength { len };
            }
            read
        }
        Reading::ArrayLength { len } => {
            let (len, is_full, read) = len.update(cursor);
            if is_full {
                let len = u64::from(len.to_u32());
                state.remaining_items = state.remaining_items - 1 + len;
                state.reading = Reading::Marker;
            } else {
                state.reading = Reading::ArrayLength { len };
            }
            read
        }
        Reading::MapLength { len } => {
            let (len, is_full, read) = len.update(cursor);
            if is_full {
                let len = u64::from(len.to_u32());
                state.remaining_items = state.remaining_items - 1 + 2 * len;
                state.reading = Reading::Marker;
            } else {
                state.reading = Reading::MapLength { len };
            }
            read
        }
        Reading::ExtLength { len } => {
            let (len, is_full, read) = len.update(cursor);
            if is_full {
                let len = u64::from(len.to_u32()) + 1;
                state.reading = Reading::CompleteBytes { len };
            } else {
                state.reading = Reading::ExtLength { len };
            }
            read
        }
    }
}
