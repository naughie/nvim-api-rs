use super::{Decode, DecodeError, DecodedArray};
use crate::handles::{Buffer, Tabpage, Window};
use crate::sys::{ext_types, types as sys};

use mp::decode::{Object, ObjectParser};

fn object(bytes: &[u8]) -> Object<&[u8]> {
    ObjectParser::new(bytes).next_object().unwrap()
}

#[test]
fn handle_extensions_and_integer_handles() {
    let buffer = <Buffer as Decode<ext_types::Buffer, _>>::decode(object(&[0xd4, 0, 7])).unwrap();
    assert_eq!(buffer, Buffer(7));
    assert_eq!(
        <Window as Decode<ext_types::Window, _>>::decode(object(&[0xd4, 1, 8])).unwrap(),
        Window(8)
    );
    assert_eq!(
        <Tabpage as Decode<ext_types::Tabpage, _>>::decode(object(&[0xd4, 2, 9])).unwrap(),
        Tabpage(9)
    );
    assert_eq!(
        <Buffer as Decode<ext_types::Buffer, _>>::decode(object(&[7])).unwrap(),
        Buffer(7)
    );
    assert_eq!(
        <Buffer as Decode<ext_types::Buffer, _>>::decode(object(&[0xd4, 1, 7])),
        Err(DecodeError::InvalidExtension {
            expected: 0,
            actual: 1
        })
    );
    assert_eq!(
        <Buffer as Decode<ext_types::Buffer, _>>::decode(object(&[0xd5, 0, 7, 8])),
        Err(DecodeError::InvalidExtensionData)
    );
    assert_eq!(
        <Buffer as Decode<ext_types::Buffer, _>>::decode(object(&[0xd4, 0, 0xcc])),
        Err(DecodeError::InvalidExtensionData)
    );
}

#[test]
fn integer_conversion_checks_the_destination_range() {
    for n in [i64::MAX as u64 + 1, u64::MAX] {
        assert_eq!(
            <u64 as Decode<sys::Int, &[u8]>>::decode(Object::U64(n)),
            Ok(n)
        );
        assert_eq!(
            <u64 as Decode<sys::Object, &[u8]>>::decode(Object::U64(n)),
            Ok(n)
        );
    }
    assert_eq!(
        <usize as Decode<sys::Int, &[u8]>>::decode(Object::U64(usize::MAX as u64)),
        Ok(usize::MAX)
    );
    assert_eq!(
        <i64 as Decode<sys::Int, &[u8]>>::decode(Object::U64(i64::MAX as u64)),
        Ok(i64::MAX)
    );
    assert_eq!(
        <i64 as Decode<sys::Int, &[u8]>>::decode(Object::I64(i64::MIN)),
        Ok(i64::MIN)
    );
    assert_eq!(
        <i64 as Decode<sys::Int, &[u8]>>::decode(Object::U64(i64::MAX as u64 + 1)),
        Err(DecodeError::IntegerOutOfRange)
    );
    assert_eq!(
        <u64 as Decode<sys::Int, &[u8]>>::decode(Object::I8(-1)),
        Err(DecodeError::IntegerOutOfRange)
    );
    assert_eq!(
        <u8 as Decode<sys::Int, &[u8]>>::decode(Object::U16(256)),
        Err(DecodeError::IntegerOutOfRange)
    );
}

fn check_integer<T>(bytes: &[u8], expected: Result<T, DecodeError>)
where
    T: std::fmt::Debug + PartialEq,
    for<'a> T: Decode<sys::Int, &'a [u8]> + Decode<sys::Object, &'a [u8]>,
{
    assert_eq!(
        <T as Decode<sys::Int, _>>::decode(object(bytes)),
        expected,
        "integer marker, {bytes:x?}"
    );
    assert_eq!(
        <T as Decode<sys::Object, _>>::decode(object(bytes)),
        expected,
        "object marker, {bytes:x?}"
    );
}

#[test]
fn integer_conversion_accepts_all_encoded_widths() {
    let encodings: &[&[u8]] = &[
        &[0xcc, 42],
        &[0xcd, 0, 42],
        &[0xce, 0, 0, 0, 42],
        &[0xcf, 0, 0, 0, 0, 0, 0, 0, 42],
        &[0xd0, 42],
        &[0xd1, 0, 42],
        &[0xd2, 0, 0, 0, 42],
        &[0xd3, 0, 0, 0, 0, 0, 0, 0, 42],
    ];
    for &bytes in encodings {
        check_integer::<i8>(bytes, Ok(42));
        check_integer::<i16>(bytes, Ok(42));
        check_integer::<i32>(bytes, Ok(42));
        check_integer::<i64>(bytes, Ok(42));
        check_integer::<isize>(bytes, Ok(42));
        check_integer::<u8>(bytes, Ok(42));
        check_integer::<u16>(bytes, Ok(42));
        check_integer::<u32>(bytes, Ok(42));
        check_integer::<u64>(bytes, Ok(42));
        check_integer::<usize>(bytes, Ok(42));
    }
}

#[test]
fn integer_conversion_preserves_negative_values_and_rejects_unsigned_outputs() {
    let encodings: &[&[u8]] = &[
        &[0xd0, 0xff],
        &[0xd1, 0xff, 0xff],
        &[0xd2, 0xff, 0xff, 0xff, 0xff],
        &[0xd3, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
    ];
    for &bytes in encodings {
        check_integer::<i8>(bytes, Ok(-1));
        check_integer::<i16>(bytes, Ok(-1));
        check_integer::<i32>(bytes, Ok(-1));
        check_integer::<i64>(bytes, Ok(-1));
        check_integer::<isize>(bytes, Ok(-1));
        check_integer::<u8>(bytes, Err(DecodeError::IntegerOutOfRange));
        check_integer::<u16>(bytes, Err(DecodeError::IntegerOutOfRange));
        check_integer::<u32>(bytes, Err(DecodeError::IntegerOutOfRange));
        check_integer::<u64>(bytes, Err(DecodeError::IntegerOutOfRange));
        check_integer::<usize>(bytes, Err(DecodeError::IntegerOutOfRange));
    }
}

#[test]
fn strings_can_be_borrowed_owned_or_raw_bytes() {
    let bytes = b"\xa5hello";
    let text = <&str as Decode<sys::String, _>>::decode(object(bytes)).unwrap();
    assert_eq!(text, "hello");
    assert_eq!(text.as_ptr(), bytes[1..].as_ptr());
    assert_eq!(
        <String as Decode<sys::String, _>>::decode(object(bytes)).unwrap(),
        "hello"
    );

    let bytes = &[0xa1, 0xff];
    assert_eq!(
        <&[u8] as Decode<sys::String, _>>::decode(object(bytes)).unwrap(),
        &[0xff]
    );
    assert_eq!(
        <Vec<u8> as Decode<sys::String, _>>::decode(object(bytes)).unwrap(),
        [0xff]
    );
    assert!(matches!(
        <&str as Decode<sys::String, _>>::decode(object(bytes)),
        Err(DecodeError::InvalidUtf8(_))
    ));
    assert!(matches!(
        <String as Decode<sys::String, _>>::decode(object(bytes)),
        Err(DecodeError::InvalidUtf8(_))
    ));
    assert_eq!(
        <&[u8] as Decode<sys::String, _>>::decode(object(&[0xc4, 1, 0xff])).unwrap(),
        &[0xff]
    );
}

#[test]
fn fixed_arrays_require_exact_length() {
    type Signature = sys::FixedArrayOf<sys::Int, 2>;
    assert_eq!(
        <[i64; 2] as Decode<Signature, _>>::decode(object(&[0x92, 1, 2])),
        Ok([1, 2])
    );
    assert_eq!(
        <(i64, i64) as Decode<Signature, _>>::decode(object(&[0x92, 1, 2])),
        Ok((1, 2))
    );
    assert_eq!(
        <[i64; 2] as Decode<Signature, _>>::decode(object(&[0x91, 1])),
        Err(DecodeError::InvalidLength {
            expected: 2,
            actual: 1
        })
    );
    assert_eq!(
        <[i64; 2] as Decode<Signature, _>>::decode(object(&[0x93, 1, 2, 3])),
        Err(DecodeError::InvalidLength {
            expected: 2,
            actual: 3
        })
    );
    assert_eq!(
        <[i64; 0] as Decode<sys::FixedArrayOf<sys::Int, 0>, _>>::decode(object(&[0x90])),
        Ok([])
    );
}

#[test]
fn fixed_array_element_errors_drop_initialized_values() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    static DROPPED: AtomicUsize = AtomicUsize::new(0);
    struct Item;
    impl Drop for Item {
        fn drop(&mut self) {
            DROPPED.fetch_add(1, Ordering::Relaxed);
        }
    }
    impl Decode<sys::Int, &[u8]> for Item {
        fn decode(value: Object<&[u8]>) -> Result<Self, DecodeError> {
            <i64 as Decode<sys::Int, _>>::decode(value)?;
            Ok(Self)
        }
    }
    let ret = <[Item; 3] as Decode<sys::FixedArrayOf<sys::Int, 3>, _>>::decode(object(&[
        0x93, 1, 2, 0xc0,
    ]));
    assert!(matches!(ret, Err(DecodeError::UnexpectedType("integer"))));
    assert_eq!(DROPPED.load(Ordering::Relaxed), 2);
}

#[test]
fn typed_arrays_decode_lazily_and_keep_accurate_lengths() {
    let mut array =
        <DecodedArray<i64, sys::Int, _> as Decode<sys::ArrayOf<sys::Int>, _>>::decode(object(&[
            0x93, 1, 0xc0, 3,
        ]))
        .unwrap();
    assert_eq!(array.len(), 3);
    assert_eq!(array.size_hint(), (3, Some(3)));
    assert_eq!(array.next(), Some(Ok(1)));
    assert_eq!(
        array.next(),
        Some(Err(DecodeError::UnexpectedType("integer")))
    );
    assert_eq!(array.len(), 1);
    assert_eq!(array.next(), Some(Ok(3)));
    assert_eq!(array.next(), None);
    assert_eq!(array.next(), None);
    assert_eq!(array.size_hint(), (0, Some(0)));
}

#[test]
fn ui_argument_tuples_accept_appended_fields() {
    type Signature = (sys::Int, sys::String);
    assert_eq!(
        <(i64, &str) as Decode<Signature, _>>::decode(object(b"\x93\x01\xa1x\xc3")),
        Ok((1, "x"))
    );
    assert_eq!(
        <(i64, &str) as Decode<Signature, _>>::decode(object(&[0x91, 1])),
        Err(DecodeError::InvalidLength {
            expected: 2,
            actual: 1
        })
    );
    assert_eq!(<() as Decode<(), _>>::decode(object(&[0x91, 1])), Ok(()));
    assert_eq!(<() as Decode<(), _>>::decode(object(&[0xc0])), Ok(()));
}
