use super::seek::State;
use super::*;
use crate::ExtType;

use rmpv::Value;

use std::num::NonZeroUsize;
use std::ops::ControlFlow;

fn encode(value: impl Into<Value>) -> Vec<u8> {
    use std::io::Cursor;

    let mut v = Vec::new();
    let mut wtr = Cursor::new(&mut v);
    let value = value.into();
    assert!(rmpv::encode::write_value(&mut wtr, &value).is_ok());
    v
}

#[test]
fn state_transition() {
    let mut state = State::new();

    let v = encode(Value::from("some long text"));

    let split = v.len() - 5;
    let (trunc, rest) = v.split_at_checked(split).unwrap();

    assert_eq!(state.transit(&mut &trunc[..]), ControlFlow::Continue(()));
    assert_eq!(state.transit(&mut &rest[..2]), ControlFlow::Continue(()));
    assert_eq!(
        state.transit(&mut &rest[..]),
        ControlFlow::Break(NonZeroUsize::new(3).unwrap())
    );
    assert_eq!(state, State::new());

    assert_eq!(
        state.transit(&mut &v[..]),
        ControlFlow::Break(NonZeroUsize::new(v.len()).unwrap())
    );
    assert_eq!(state, State::new());

    assert_eq!(state.transit(&mut &b""[..]), ControlFlow::Continue(()));

    let mut v = [rmp::Marker::Str8.to_u8(), 50].to_vec();
    v.extend_from_slice(b"invalid");
    assert_eq!(state.transit(&mut &v[..]), ControlFlow::Continue(()));
    assert_eq!(state.transit(&mut &b""[..]), ControlFlow::Continue(()));

    let v = [b'a'; 43];
    assert_eq!(
        state.transit(&mut &v[..]),
        ControlFlow::Break(NonZeroUsize::new(v.len()).unwrap())
    );
    assert_eq!(state, State::new());
}

fn seek_eq(buf: &mut impl Buf, expected: Option<usize>) {
    let expected = expected
        .map(|n| NonZeroUsize::new(n).unwrap())
        .ok_or(InsufficientBytes);
    assert_eq!(seek_to_eoo(buf), expected);
}

#[test]
fn seek_valid() {
    fn assert_impl(value: impl Into<Value>) {
        let v = encode(value);
        seek_eq(&mut v.as_slice(), Some(v.len()));
    }

    assert_impl(Value::Nil);
    assert_impl(true);
    assert_impl(false);
    assert_impl(0x41u8);
    assert_impl(0xc0u8);
    assert_impl(1i64);
    assert_impl(1.0f32);
    assert_impl(1.0f64);

    assert_impl("");
    assert_impl("foo bar");
    assert_impl("non-ASCII: 漢字");
    assert_impl(b"".to_vec());
    assert_impl(b"foo bar".to_vec());

    assert_impl(Vec::<Value>::new());
    assert_impl(vec![Value::from(1i32)]);
    assert_impl(vec![Value::from(1i32), Value::from("string")]);

    assert_impl(Vec::<(Value, Value)>::new());
    assert_impl(vec![(Value::from("key1"), Value::from("val1"))]);
    assert_impl(vec![
        (Value::from("key1"), Value::from("val1")),
        (Value::from("key2"), Value::from(2)),
    ]);

    assert_impl(Value::Ext(8, vec![]));
    assert_impl(Value::Ext(8, b"ext".to_vec()));

    assert_impl(vec![Value::Array(vec![
        Value::Array(vec![]),
        Value::from(1),
        Value::from(2),
        Value::Array(vec![Value::Map(vec![
            (Value::from(1), Value::from(2)),
            (Value::from(2), Value::Array(vec![Value::from(1)])),
        ])]),
    ])]);
}

#[test]
fn seek_invalid() {
    let v = encode(Value::from("some long text"));

    let split = v.len() - 5;
    let (trunc, rest) = v.split_at_checked(split).unwrap();
    let mut trunc = trunc.to_vec();

    assert!(seek_to_eoo(&mut trunc.as_slice()).is_err());

    trunc.extend_from_slice(&rest[..2]);
    assert!(seek_to_eoo(&mut trunc.as_slice()).is_err());

    trunc.extend_from_slice(&rest[2..]);
    trunc.push(rmp::Marker::Str8.to_u8());
    trunc.push(50);
    trunc.extend_from_slice(b"invalid");
    seek_eq(&mut trunc.as_slice(), Some(v.len()));
    seek_eq(&mut &trunc[v.len()..], None);

    trunc.extend_from_slice(&[b'a'; 43][..]);
    seek_eq(&mut &trunc[v.len()..], Some(52));
}

#[test]
fn seek_sequence() {
    let mut buf = Vec::new();
    let mut lengths: Vec<usize> = Vec::new();

    let values = [
        Value::from(1i32),
        Value::from("string"),
        Value::Nil,
        Value::from(vec![Value::from(2), Value::from(3)]),
    ];

    for value in values {
        let v = encode(value);
        lengths.push(v.len());
        buf.extend(v);
    }

    let mut buf = buf.as_slice();
    let cursor = &mut buf;
    seek_eq(cursor, Some(lengths[0]));
    seek_eq(cursor, Some(lengths[1]));
    seek_eq(cursor, Some(lengths[2]));
    seek_eq(cursor, Some(lengths[3]));
    assert!(cursor.is_empty());
}

#[test]
fn parse() {
    use rmp::Marker;

    let mut buf = Vec::new();

    let values = [
        Value::from(1u8),
        Value::from(-1i8),
        Value::from(1.0f32),
        Value::from(1.5f64),
        Value::from("string"),
        Value::Binary(b"binary".to_vec()),
        Value::Nil,
        Value::from(true),
        Value::from(false),
        Value::from(vec![
            Value::from(2),
            Value::from(300),
            Value::from(i32::MIN),
            Value::from(i64::MIN),
        ]),
        Value::from(vec![
            (Value::from("key1"), Value::from("value1")),
            (Value::from("key2"), Value::from("value2")),
        ]),
        Value::Ext(5, b"ext".to_vec()),
    ];

    for value in values {
        let v = encode(value);
        buf.extend(v);
    }
    let remains = &[Marker::Map16.to_u8(), 2];
    buf.extend(remains);

    let mut parser = ObjectParser::new(&buf[..]);

    let obj = parser.next().unwrap();
    assert!(matches!(obj, Object::U8(1)));

    let obj = parser.next().unwrap();
    assert!(matches!(obj, Object::I8(-1)));

    let obj = parser.next().unwrap();
    assert!(matches!(obj, Object::F32(1.0)));

    let obj = parser.next().unwrap();
    assert!(matches!(obj, Object::F64(1.5)));

    let obj = parser.next().unwrap();
    if let Object::Str(s) = obj {
        assert_eq!(s.as_slice(), b"string");
    } else {
        panic!("obj is not Object::Str");
    }

    let obj = parser.next().unwrap();
    if let Object::Bin(s) = obj {
        assert_eq!(s.as_slice(), b"binary");
    } else {
        panic!("obj is not Object::Bin");
    }

    let obj = parser.next().unwrap();
    assert!(matches!(obj, Object::Nil));

    let obj = parser.next().unwrap();
    assert!(matches!(obj, Object::Bool(true)));
    let obj = parser.next().unwrap();
    assert!(matches!(obj, Object::Bool(false)));

    {
        let obj = parser.next().unwrap();
        if let Object::Array(mut arr) = obj {
            let obj = arr.next().unwrap();
            assert!(matches!(obj, Object::U8(2)));

            let obj = arr.next().unwrap();
            assert!(matches!(obj, Object::U16(300)));

            let obj = arr.next().unwrap();
            assert!(matches!(obj, Object::I32(i32::MIN)));

            let obj = arr.next().unwrap();
            assert!(matches!(obj, Object::I64(i64::MIN)));

            assert!(arr.next().is_none());
        } else {
            panic!("obj is not Object::Array");
        }
    }

    {
        let obj = parser.next().unwrap();
        if let Object::Map(mut arr) = obj {
            let item = arr.next().unwrap();
            if let Object::Str(s) = item.key {
                assert_eq!(s.as_slice(), b"key1");
            } else {
                panic!("obj is not Object::Str");
            }
            if let Object::Str(s) = item.value {
                assert_eq!(s.as_slice(), b"value1");
            } else {
                panic!("obj is not Object::Str");
            }

            let item = arr.next().unwrap();
            if let Object::Str(s) = item.key {
                assert_eq!(s.as_slice(), b"key2");
            } else {
                panic!("obj is not Object::Str");
            }
            if let Object::Str(s) = item.value {
                assert_eq!(s.as_slice(), b"value2");
            } else {
                panic!("obj is not Object::Str");
            }

            assert!(arr.next().is_none());
        } else {
            panic!("obj is not Object::Map");
        }
    }

    let obj = parser.next().unwrap();
    if let Object::Ext(id, data) = obj {
        assert_eq!(id, ExtType(5));
        assert_eq!(data.as_slice(), b"ext");
    } else {
        panic!("obj is not Object::Ext");
    }

    assert!(parser.next().is_none());
    assert_eq!(parser.remaining(), remains);
}

#[test]
fn slice_cursor_bounds_and_consumption() {
    use super::buf::SliceCursor;
    use std::ops::Bound;

    let bytes = [1, 2, 3, 4];
    let mut buf = SliceCursor::new(&bytes[..], 1..3);
    let mut dst = [0; 2];
    buf.copy_to_slice(&mut dst);
    assert_eq!(dst, [2, 3]);
    assert_eq!(buf.len(), 0);
    assert_eq!(buf.first_byte(), None);

    let mut buf = SliceCursor::new(&bytes[..], (Bound::Excluded(1), Bound::Included(2)));
    assert_eq!(buf.first_byte(), Some(3));
    assert_eq!(buf.first_byte(), None);

    let mut buf = SliceCursor::new(&bytes[..], 0..0);
    assert_eq!(buf.first_byte(), None);
    assert_eq!(buf.slice(..0).len(), 0);
    assert_eq!(SliceCursor::new(&[][..], ..).len(), 0);

    let mut buf = SliceCursor::new(&bytes[..], 3..=usize::MAX);
    assert_eq!(buf.first_byte(), Some(4));
    assert_eq!(buf.first_byte(), None);
}

fn assert_array16(buf: impl Buf) {
    let mut parser = ObjectParser::new(buf);
    let Object::Array(mut array) = parser.next_object().unwrap() else {
        panic!("expected an array");
    };
    assert!(matches!(array.next(), Some(Object::U16(0x1234))));
    assert!(matches!(array.next(), Some(Object::U16(0x5678))));
    assert!(array.next().is_none());
    assert!(parser.next().is_none());
}

#[test]
fn parse_noncontiguous_array16() {
    use super::buf::SliceCursor;
    use std::collections::VecDeque;

    let bytes = [0xdc, 0, 2, 0xcd, 0x12, 0x34, 0xcd, 0x56, 0x78];
    let mut buf = VecDeque::with_capacity(bytes.len());
    buf.extend([0; 4]);
    for _ in 0..4 {
        buf.pop_front();
    }
    buf.extend(bytes);
    assert_array16(SliceCursor::new(&buf, ..));
}

#[cfg(feature = "bytes")]
#[test]
fn parse_bytes_array16() {
    assert_array16(bytes::Bytes::from_static(&[
        0xdc, 0, 2, 0xcd, 0x12, 0x34, 0xcd, 0x56, 0x78,
    ]));
}

fn assert_borrowed_array_view<B: Buf<Slice = B> + AsRef<[u8]>>(buf: B) {
    let Object::Array(mut array) = ObjectParser::new(buf).next_object().unwrap() else {
        panic!("expected an array");
    };
    assert_eq!(array.next().unwrap().to_u8(), Some(1));
    assert_eq!(array.remaining_len(), 3);

    let text_ptr = {
        let mut view: Array<&[u8]> = array.as_borrowed();
        assert_eq!(view.remaining_len(), 3);
        assert_eq!(view.size_hint(), (3, Some(3)));
        let Object::Str(text) = view.next().unwrap() else {
            panic!("expected a string");
        };
        assert_eq!(text.as_slice(), b"two");
        assert_eq!(view.remaining_len(), 2);
        assert_eq!(array.remaining_len(), 3);

        let Object::Array(mut nested) = view.next().unwrap() else {
            panic!("expected a nested array");
        };
        assert_eq!(nested.next().unwrap().to_u8(), Some(3));
        assert_eq!(nested.next().unwrap().to_bool(), Some(false));
        assert!(nested.next().is_none());
        assert_eq!(view.next().unwrap().to_bool(), Some(true));
        assert_eq!(view.remaining_len(), 0);
        assert!(view.next().is_none());
        assert!(view.next().is_none());
        assert_eq!(array.remaining_len(), 3);
        text.as_slice().as_ptr()
    };

    let Object::Str(text) = array.next().unwrap() else {
        panic!("expected the original string");
    };
    assert_eq!(text.as_slice(), b"two");
    assert_eq!(text.as_slice().as_ptr(), text_ptr);
    assert_eq!(array.as_borrowed().remaining_len(), 2);

    let Object::Array(mut nested) = array.next().unwrap() else {
        panic!("expected the original nested array");
    };
    assert_eq!(nested.next().unwrap().to_u8(), Some(3));
    assert_eq!(nested.next().unwrap().to_bool(), Some(false));
    assert!(nested.next().is_none());
    assert_eq!(array.next().unwrap().to_bool(), Some(true));
    assert!(array.next().is_none());
    let mut empty = array.as_borrowed();
    assert_eq!(empty.remaining_len(), 0);
    assert_eq!(empty.size_hint(), (0, Some(0)));
    assert!(empty.next().is_none());
}

#[test]
fn borrowed_array_view_keeps_position_order_and_storage() {
    assert_borrowed_array_view(&b"\x94\x01\xa3two\x92\x03\xc2\xc3"[..]);
}

#[cfg(feature = "bytes")]
#[test]
fn borrowed_bytes_array_view_keeps_position_order_and_storage() {
    assert_borrowed_array_view(bytes::Bytes::from_static(
        b"\x94\x01\xa3two\x92\x03\xc2\xc3",
    ));
}

#[test]
fn seek_large_lengths_remain_incomplete() {
    for bytes in [
        &[0xdf, 0xff, 0xff, 0xff, 0xff][..],
        &[0xdd, 0xff, 0xff, 0xff, 0xff, 0x92][..],
        &[0xdd, 0xff, 0xff, 0xff, 0xff, 0xdd, 0xff, 0xff, 0xff, 0xff][..],
        &[0xc9, 0xff, 0xff, 0xff, 0xff][..],
    ] {
        assert_eq!(seek_to_eoo(&mut &bytes[..]), Err(InsufficientBytes));
    }
}

#[test]
fn seek_state_preserves_fragmented_headers_and_trailing_objects() {
    let bytes = [0x92, 0xa3, b'f', b'o', b'o', 0xdc, 0, 2, 1, 2, 0xc0];
    let object_len = bytes.len() - 1;
    for split in 0..object_len {
        let mut state = SeekState::new();
        let mut first = &bytes[..split];
        assert_eq!(state.transit(&mut first), ControlFlow::Continue(()));
        assert!(first.is_empty());

        let mut rest = &bytes[split..];
        assert_eq!(
            state.transit(&mut rest),
            ControlFlow::Break(NonZeroUsize::new(object_len - split).unwrap())
        );
        assert_eq!(rest, &[0xc0]);
        assert_eq!(state, SeekState::new());
        assert_eq!(
            state.transit(&mut rest),
            ControlFlow::Break(NonZeroUsize::new(1).unwrap())
        );
        assert!(rest.is_empty());
    }
}
