use nvim_rpc::api_types::BaseType;
use nvim_rpc::api_types::encode::{Convert, Like, Untagged};
use nvim_rpc::mp::encode::primitive::{Array, ArrayIter, Map, MapIter, Nil, State as _};
use nvim_rpc::mp::encode::{Encode as _, WriterSyncExt as _};
use nvim_rpc::sys::types::{self, OneOf};
use nvim_rpc::{Neovim, api};

use std::cell::Cell;

type ValueType = OneOf<types::Int, types::String>;

enum UserValue {
    Number(i32),
    Text(String),
}

impl Like<ValueType> for UserValue {
    type EncodeAs = Untagged<i64, String>;

    fn encode(self) -> Self::EncodeAs {
        match self {
            Self::Number(n) => Untagged::Left(i64::from(n)),
            Self::Text(s) => Untagged::Right(s),
        }
    }
}

impl<'s> Like<ValueType> for &'s UserValue {
    type EncodeAs = Untagged<i64, &'s str>;

    fn encode(self) -> Self::EncodeAs {
        match self {
            UserValue::Number(n) => Untagged::Left(i64::from(*n)),
            UserValue::Text(s) => Untagged::Right(s.as_str()),
        }
    }
}

fn assert_encoded<B: BaseType>(value: impl Like<B>, expected: &[u8]) {
    let mut bytes = Vec::new();
    bytes.write_value(<_ as Like<B>>::encode(value)).consume();
    assert_eq!(bytes, expected);
}

#[test]
fn custom_enums_encode_the_selected_payload_without_a_tag() {
    assert_encoded::<ValueType>(UserValue::Number(-1), b"\xff");
    assert_encoded::<ValueType>(UserValue::Text("text".into()), b"\xa4text");

    let value = UserValue::Text("borrowed".into());
    let Untagged::Right(text) = <&UserValue as Like<ValueType>>::encode(&value) else {
        panic!("expected text");
    };
    let UserValue::Text(source) = &value else {
        unreachable!();
    };
    assert_eq!(text.as_ptr(), source.as_ptr());
    assert_encoded::<ValueType>(&value, b"\xa8borrowed");

    let source = String::from("moved");
    let ptr = source.as_ptr();
    let Untagged::Right(text) = <UserValue as Like<ValueType>>::encode(UserValue::Text(source))
    else {
        panic!("expected text");
    };
    assert_eq!(text.as_ptr(), ptr);
}

#[test]
fn a_narrow_conversion_can_supply_an_object_argument_explicitly() {
    let value = UserValue::Text("text".into());
    let encoded = <&UserValue as Like<ValueType>>::encode(&value);
    let mut nvim = Neovim::new(Vec::new());
    nvim.notify::<api::NvimSetVar>(("value", encoded)).unwrap();
    assert_eq!(
        nvim.into_inner(),
        b"\x93\x02\xacnvim_set_var\x92\xa5value\xa4text",
    );
}

#[test]
fn nesting_supports_three_through_six_alternatives() {
    use Untagged::{Left, Right};

    type K3 = OneOf<types::Nil, OneOf<types::Bool, types::Int>>;
    type E3 = Untagged<Nil, Untagged<bool, i64>>;
    type K4 = OneOf<K3, types::String>;
    type E4 = Untagged<E3, &'static str>;
    type K5 = OneOf<K4, types::Float>;
    type E5 = Untagged<E4, f64>;
    type K6 = OneOf<K5, types::Array>;
    type E6 = Untagged<E5, Array<[u8; 1]>>;

    assert_encoded::<K3>(E3::Left(Nil), b"\xc0");
    assert_encoded::<K3>(E3::Right(Left(true)), b"\xc3");
    assert_encoded::<K3>(E3::Right(Right(7)), b"\x07");
    assert_encoded::<K4>(E4::Right("s"), b"\xa1s");
    assert_encoded::<K5>(E5::Right(1.5), b"\xcb\x3f\xf8\0\0\0\0\0\0");

    let values: [E6; 6] = [
        Left(Left(Left(Left(Nil)))),
        Left(Left(Left(Right(Left(true))))),
        Left(Left(Left(Right(Right(7))))),
        Left(Left(Right("s"))),
        Left(Right(1.5)),
        Right(Array([9])),
    ];
    let expected: [&[u8]; 6] = [
        b"\xc0",
        b"\xc3",
        b"\x07",
        b"\xa1s",
        b"\xcb\x3f\xf8\0\0\0\0\0\0",
        b"\x91\x09",
    ];
    for (value, expected) in values.into_iter().zip(expected) {
        assert_encoded::<K6>(value, expected);
    }

    assert_encoded::<OneOf<types::Int, types::Dict>>(
        Untagged::<i64, _>::Right(Map([("key", 7u8)])),
        b"\x81\xa3key\x07",
    );
}

#[test]
fn alternatives_can_share_a_marker_and_be_borrowed() {
    type Integers = Untagged<i32, Untagged<i64, i16>>;

    assert_encoded::<types::Int>(Integers::Left(7), b"\x07");
    assert_encoded::<types::Int>(Integers::Right(Untagged::Left(9)), b"\x09");
    let value = Integers::Right(Untagged::Right(-1));
    assert_encoded::<types::Int>(&value, b"\xff");
    assert_encoded::<types::Object>(&value, b"\xff");

    let value: Untagged<i64, &str> = Untagged::Right("s");
    assert_encoded::<ValueType>(&value, b"\xa1s");
}

#[test]
fn collections_accept_alternatives_and_custom_union_conversions() {
    let values = [UserValue::Number(7), UserValue::Text("s".into())];
    let expected = b"\x92\x07\xa1s";
    assert_encoded::<types::ArrayOf<ValueType>>(&values[..], expected);
    assert_encoded::<types::FixedArrayOf<ValueType, 2>>(&values, expected);
    assert_encoded::<types::ArrayOf<ValueType>>(Convert(Array(&values)), expected);
    assert_encoded::<types::ArrayOf<ValueType>>(Convert(ArrayIter(values.iter())), expected);
    assert_encoded::<types::FixedArrayOf<ValueType, 2>>(
        Convert((&values[0], &values[1])),
        expected,
    );

    let encoded = <&UserValue as Like<ValueType>>::encode(&values[1]);
    assert_encoded::<types::Dict>(Map([("key", encoded)]), b"\x81\xa3key\xa1s");
    let encoded = <&UserValue as Like<ValueType>>::encode(&values[0]);
    assert_encoded::<types::Dict>(MapIter([("key", encoded)].into_iter()), b"\x81\xa3key\x07");

    let encoded = [
        <&UserValue as Like<ValueType>>::encode(&values[0]),
        <&UserValue as Like<ValueType>>::encode(&values[1]),
    ];
    assert_encoded::<types::ArrayOf<ValueType>>(Array(encoded), expected);
    let encoded = <&UserValue as Like<ValueType>>::encode(&values[0]);
    assert_encoded::<types::ArrayOf<ValueType>>((encoded,), b"\x91\x07");

    let empty: [UserValue; 0] = [];
    assert_encoded::<types::ArrayOf<ValueType>>(&empty[..], b"\x90");
}

struct Counted<'s>(&'s Cell<usize>, &'s UserValue);

impl<'s> Like<ValueType> for Counted<'s> {
    type EncodeAs = Untagged<i64, &'s str>;

    fn encode(self) -> Self::EncodeAs {
        self.0.set(self.0.get() + 1);
        <&UserValue as Like<ValueType>>::encode(self.1)
    }
}

impl<'s> Like<types::Object> for Counted<'s> {
    type EncodeAs = Untagged<i64, &'s str>;

    fn encode(self) -> Self::EncodeAs {
        <Self as Like<ValueType>>::encode(self)
    }
}

#[test]
fn collection_conversion_remains_lazy_with_union_elements() {
    let count = Cell::new(0);
    let values = [UserValue::Number(7), UserValue::Text("s".into())];
    let encoded = <_ as Like<types::ArrayOf<ValueType>>>::encode(Convert(ArrayIter(
        values.iter().map(|value| Counted(&count, value)),
    )));
    let mut state = encoded.into_encoder();
    assert_eq!(count.get(), 0);
    assert_eq!(state.next().unwrap().as_ref(), b"\x92");
    assert_eq!(count.get(), 0);
    assert_eq!(state.next().unwrap().as_ref(), b"\x07");
    assert_eq!(count.get(), 1);
    assert_eq!(state.next().unwrap().as_ref(), b"\xa1");
    assert_eq!(count.get(), 2);
    assert_eq!(state.next().unwrap().as_ref(), b"s");
    assert!(state.next().is_none());

    count.set(0);
    let encoded = <_ as Like<types::Dict>>::encode(Convert(MapIter(
        values.iter().map(|value| ("key", Counted(&count, value))),
    )));
    let mut state = encoded.into_encoder();
    assert_eq!(state.next().unwrap().as_ref(), b"\x82");
    assert_eq!(count.get(), 0);
    let mut bytes = Vec::new();
    while let Some(chunk) = state.next() {
        bytes.extend_from_slice(chunk.as_ref());
    }
    assert_eq!(bytes, b"\xa3key\x07\xa3key\xa1s");
    assert_eq!(count.get(), 2);
}
