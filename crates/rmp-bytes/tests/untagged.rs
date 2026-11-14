use rmp_bytes::encode::primitive::{Array, ArrayIter, Map, MapIter, State as _};
use rmp_bytes::encode::{Encode, Untagged, WriterAsyncExt, WriterSyncExt};

fn assert_encoded(value: impl Encode, expected: &[u8]) {
    let mut bytes = Vec::new();
    WriterSyncExt::write_value(&mut bytes, value).consume();
    assert_eq!(bytes, expected);
}

#[test]
fn alternatives_encode_only_the_selected_payload() {
    assert_encoded(Untagged::<i64, String>::Left(-7), b"\xf9");
    assert_encoded(Untagged::<i64, String>::Right("text".into()), b"\xa4text");

    type Value = Untagged<i64, Untagged<String, Array<[bool; 2]>>>;
    assert_encoded(Value::Left(7), b"\x07");
    assert_encoded(Value::Right(Untagged::Left("text".into())), b"\xa4text");
    assert_encoded(
        Value::Right(Untagged::Right(Array([true, false]))),
        b"\x92\xc3\xc2",
    );
}

#[test]
fn borrowed_alternatives_retain_the_source_and_payload_borrow() {
    let value: Untagged<i64, String> = Untagged::Right("text".into());
    let Untagged::Right(text) = &value else {
        panic!("expected text");
    };
    let mut state = (&value).into_encoder();
    assert_eq!(state.next().unwrap().as_ref(), b"\xa4");
    let chunk = state.next().unwrap();
    assert_eq!(chunk.as_ref().as_ptr(), text.as_ptr());
    assert_eq!(chunk.as_ref(), b"text");
    assert!(state.next().is_none());

    assert_encoded(&value, b"\xa4text");
    assert_encoded(&value, b"\xa4text");
}

#[test]
fn primitive_containers_accept_composed_elements() {
    let values = [Untagged::Left(7i64), Untagged::Right("text")];
    assert_encoded(Array(&values), b"\x92\x07\xa4text");
    assert_encoded(ArrayIter(values.into_iter()), b"\x92\x07\xa4text");
    assert_encoded((values[0], values[1]), b"\x92\x07\xa4text");

    assert_encoded(
        Map([("a", values[0]), ("b", values[1])]),
        b"\x82\xa1a\x07\xa1b\xa4text",
    );
    assert_encoded(
        MapIter([("a", values[0]), ("b", values[1])].into_iter()),
        b"\x82\xa1a\x07\xa1b\xa4text",
    );
    assert_encoded(Map([(values[0], values[1])]), b"\x81\x07\xa4text");
}

#[tokio::test]
async fn asynchronous_writes_accept_composed_representations() {
    let value: Untagged<i64, &str> = Untagged::Right("text");
    let mut bytes = Vec::new();
    WriterAsyncExt::write_value(&mut bytes, value)
        .consume_async()
        .await;
    assert_eq!(bytes, b"\xa4text");
}

#[cfg(feature = "rpc")]
#[tokio::test]
async fn response_fields_accept_alternatives_with_explicit_nil_slots() {
    use rmp_bytes::encode::rpc::{MessageId, Response};

    type Value = Untagged<i64, String>;
    let mut bytes = Vec::new();
    WriterSyncExt::write_primitive(
        &mut bytes,
        Response {
            msgid: MessageId(7),
            result: Ok::<_, Value>(Value::Right("text".into())),
        }
        .to_primitive(),
    )
    .consume();
    assert_eq!(bytes, b"\x94\x01\x07\xc0\xa4text");

    bytes.clear();
    WriterAsyncExt::write_primitive(
        &mut bytes,
        Response {
            msgid: MessageId(7),
            result: Err::<Value, _>(Value::Left(-7)),
        }
        .to_primitive(),
    )
    .consume_async()
    .await;
    assert_eq!(bytes, b"\x94\x01\x07\xf9\xc0");
}
