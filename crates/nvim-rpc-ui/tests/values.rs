use nvim_rpc::mp::encode::primitive::Map;
use nvim_rpc::mp::encode::{Primitive, WriterSyncExt as _};
use nvim_rpc::rpc::Message;

use nvim_rpc_ui::{Event, Frame, Limits, Model, ProtocolError, Value};

use std::sync::Arc;

fn encode(params: impl Primitive) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.write_primitive((2u8, "redraw", params)).consume();
    bytes
}

fn apply(model: &mut Model, bytes: &[u8]) -> Result<Vec<Frame>, ProtocolError> {
    let Message::Notification(notification) = nvim_rpc::rpc::decode(&mut &bytes[..]).unwrap()
    else {
        panic!("expected redraw notification");
    };
    let mut frames = Vec::new();
    model.apply_redraw(notification, |frame| frames.push(frame))?;
    Ok(frames)
}

#[test]
fn cloned_nested_values_share_storage_and_maps_keep_duplicate_keys_in_order() {
    let mut model = Model::default();
    let frames = apply(
        &mut model,
        &encode((
            (
                "option_set",
                ("future", (Map([("same", 1), ("same", 2)]), (9, 10))),
            ),
            ("flush", ()),
        )),
    )
    .unwrap();
    let value = &frames[0].state.options["future"];
    let clone = value.clone();
    let (Value::Array(original), Value::Array(clone)) = (value, &clone) else {
        panic!("expected array values");
    };
    assert!(Arc::ptr_eq(original, clone));

    let map = original[0].clone();
    let (Value::Map(original_map), Value::Map(cloned_map)) = (&original[0], map) else {
        panic!("expected map values");
    };
    assert!(Arc::ptr_eq(original_map, &cloned_map));
    assert_eq!(
        original_map.as_ref(),
        [
            (Value::String(Arc::from(&b"same"[..])), Value::Int(1)),
            (Value::String(Arc::from(&b"same"[..])), Value::Int(2)),
        ]
    );

    let nested = original[1].clone();
    let (Value::Array(original_nested), Value::Array(cloned_nested)) = (&original[1], nested)
    else {
        panic!("expected nested arrays");
    };
    assert!(Arc::ptr_eq(original_nested, &cloned_nested));
    assert_eq!(original_nested.as_ref(), [Value::Int(9), Value::Int(10)]);
}

#[test]
fn retained_strings_preserve_bytes_and_remain_distinct_from_binary() {
    let mut model = Model::default();
    let frames = apply(
        &mut model,
        &encode((
            (
                "option_set",
                ("text", "日本語\0"),
                ("binary", "日本語\0".as_bytes()),
            ),
            ("flush", ()),
        )),
    )
    .unwrap();
    let options = &frames[0].state.options;
    let Value::String(text) = &options["text"] else {
        panic!("expected a string value");
    };
    let Value::Binary(binary) = &options["binary"] else {
        panic!("expected a binary value");
    };
    assert_eq!(&**text, "日本語\0".as_bytes());
    assert_eq!(&**binary, "日本語\0".as_bytes());
    assert_ne!(options["text"], options["binary"]);

    let cloned = options["text"].clone();
    let Value::String(cloned) = cloned else {
        panic!("expected a string value");
    };
    assert!(Arc::ptr_eq(text, &cloned));
}

#[test]
fn unknown_events_and_highlight_fields_retain_non_utf8_string_data() {
    let mut model = Model::default();
    let frames = apply(
        &mut model,
        b"\x93\x02\xa6redraw\x92\x92\xacfuture_event\x91\xa1\xff\x92\xa5flush\x90",
    )
    .unwrap();
    assert!(!model.is_failed());
    assert!(matches!(
        &frames[0].events[..],
        [Event::Unknown { name, args }]
            if &**name == "future_event"
                && args == &[Value::String(Arc::from(&b"\xff"[..]))]
    ));

    let mut bytes = encode((
        (
            "hl_attr_define",
            (1, Map([("future", "X")]), Map([] as [(&str, i32); 0]), ()),
        ),
        ("flush", ()),
    ));
    let idx = bytes.windows(2).position(|v| v == [0xa1, b'X']).unwrap();
    bytes[idx + 1] = 0xff;
    let frames = apply(&mut model, &bytes).unwrap();
    assert!(!model.is_failed());
    assert_eq!(
        frames[0].state.highlights[&nvim_rpc_ui::HighlightId(1)]
            .rgb
            .extra["future"],
        Value::String(Arc::from(&b"\xff"[..]))
    );
}

#[test]
fn retained_values_obey_string_collection_and_nesting_limits() {
    fn rejects(params: impl Primitive, limits: Limits, expected: &str) {
        let mut model = Model::new(limits);
        let result = apply(&mut model, &encode(params));
        assert!(matches!(result, Err(ProtocolError::Limit(name)) if name == expected));
        assert!(model.is_failed());
    }

    rejects(
        (
            ("option_set", ("future", "abcdefghijklmnopq")),
            ("flush", ()),
        ),
        Limits {
            max_string_bytes: 16,
            ..Limits::default()
        },
        "string bytes",
    );
    rejects(
        (("option_set", ("future", (1, 2))), ("flush", ())),
        Limits {
            max_collection_len: 1,
            ..Limits::default()
        },
        "collection length",
    );
    rejects(
        (
            ("option_set", ("future", Map([("one", 1), ("two", 2)]))),
            ("flush", ()),
        ),
        Limits {
            max_collection_len: 1,
            ..Limits::default()
        },
        "collection length",
    );
    rejects(
        (("option_set", ("future", ((1,),))), ("flush", ())),
        Limits {
            max_depth: 1,
            ..Limits::default()
        },
        "value depth",
    );
}
