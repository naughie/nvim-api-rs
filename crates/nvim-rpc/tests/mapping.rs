use nvim_rpc::api_types::BaseType;
use nvim_rpc::api_types::encode::{Like, Map, MapRef};
use nvim_rpc::mp::encode::WriterSyncExt as _;
use nvim_rpc::mp::encode::primitive::{Array, ArrayIter, Primitive as _, State as _};
use nvim_rpc::sys::types;
use nvim_rpc::{Buffer, Neovim, Window, api};

use std::cell::Cell;

struct Line(String);

impl Like<types::String> for Line {
    type EncodeAs = String;

    fn encode(self) -> Self::EncodeAs {
        self.0
    }
}

impl Like<types::Object> for Line {
    type EncodeAs = String;

    fn encode(self) -> Self::EncodeAs {
        self.0
    }
}

impl<'s> Like<types::String> for &'s Line {
    type EncodeAs = &'s str;

    fn encode(self) -> Self::EncodeAs {
        &self.0
    }
}

fn assert_encoded<B: BaseType>(value: impl Like<B>, expected: &[u8]) {
    let mut bytes = Vec::new();
    bytes.write_value(<_ as Like<B>>::encode(value)).consume();
    assert_eq!(bytes, expected);
}

#[test]
fn mapping_preserves_exact_lengths_without_running_the_callback() {
    let count = Cell::new(0);
    let encoded =
        <_ as Like<types::ArrayOf<types::String>>>::encode(Map(["first", "second"], |s| {
            count.set(count.get() + 1);
            s
        }));
    let mut it = encoded.0.into_iter();
    assert_eq!(count.get(), 0);
    assert_eq!(it.len(), 2);
    assert_eq!(it.size_hint(), (2, Some(2)));
    assert_eq!(count.get(), 0);
    assert_eq!(it.next(), Some("first"));
    assert_eq!(count.get(), 1);
    assert_eq!(it.len(), 1);
    assert_eq!(it.size_hint(), (1, Some(1)));
    assert_eq!(it.next(), Some("second"));
    assert_eq!(count.get(), 2);
    assert_eq!(it.len(), 0);
    assert_eq!(it.size_hint(), (0, Some(0)));
    assert_eq!(it.next(), None);
    assert_eq!(count.get(), 2);
}

#[test]
fn encoding_maps_once_per_element_after_writing_the_array_header() {
    struct Counted<'s>(&'s Cell<usize>, &'s str);

    impl<'s> Like<types::String> for Counted<'s> {
        type EncodeAs = &'s str;

        fn encode(self) -> Self::EncodeAs {
            self.0.set(self.0.get() + 1);
            self.1
        }
    }

    let mapped = Cell::new(0);
    let converted = Cell::new(0);
    let encoded =
        <_ as Like<types::ArrayOf<types::String>>>::encode(Map(["a", "bc"].into_iter(), |s| {
            mapped.set(mapped.get() + 1);
            Counted(&converted, s)
        }));
    let mut state = encoded.into_state();
    assert_eq!((mapped.get(), converted.get()), (0, 0));
    assert_eq!(state.next().unwrap().as_ref(), b"\x92");
    assert_eq!((mapped.get(), converted.get()), (0, 0));
    assert_eq!(state.next().unwrap().as_ref(), b"\xa1");
    assert_eq!((mapped.get(), converted.get()), (1, 1));
    assert_eq!(state.next().unwrap().as_ref(), b"a");
    assert_eq!((mapped.get(), converted.get()), (1, 1));
    assert_eq!(state.next().unwrap().as_ref(), b"\xa2");
    assert_eq!((mapped.get(), converted.get()), (2, 2));
    assert_eq!(state.next().unwrap().as_ref(), b"bc");
    assert!(state.next().is_none());
    assert_eq!((mapped.get(), converted.get()), (2, 2));
}

#[test]
fn callbacks_move_non_clone_inputs_and_convert_custom_outputs() {
    let lines = [Line("a".into()), Line("bc".into())];
    assert_encoded::<types::ArrayOf<types::String>>(Map(lines, |Line(s)| s), b"\x92\xa1a\xa2bc");
    assert_encoded::<types::ArrayOf<types::String>>(
        Map([String::from("a"), String::from("bc")], Line),
        b"\x92\xa1a\xa2bc",
    );
    assert_encoded::<types::Array>(
        Map([String::from("a"), String::from("bc")], Line),
        b"\x92\xa1a\xa2bc",
    );
    assert_encoded::<types::Object>(
        Map([String::from("a"), String::from("bc")], Line),
        b"\x92\xa1a\xa2bc",
    );

    let lines = [Line("a".into()), Line("bc".into())];
    assert_encoded::<types::ArrayOf<types::String>>(
        MapRef(&lines, std::convert::identity),
        b"\x92\xa1a\xa2bc",
    );
}

#[test]
fn borrowed_mapping_preserves_payload_storage_and_reuses_the_source() {
    let lines = [String::from("a"), String::from("bc")];
    let encoded =
        <_ as Like<types::ArrayOf<types::String>>>::encode(MapRef(&lines[..], String::as_str));
    let mut state = encoded.into_state();
    assert_eq!(state.next().unwrap().as_ref(), b"\x92");
    for line in &lines {
        state.next().unwrap();
        let chunk = state.next().unwrap();
        assert_eq!(chunk.as_ref(), line.as_bytes());
        assert_eq!(chunk.as_ref().as_ptr(), line.as_ptr());
    }
    assert!(state.next().is_none());

    assert_encoded::<types::ArrayOf<types::String>>(
        MapRef(&lines[..], String::as_str),
        b"\x92\xa1a\xa2bc",
    );
    assert_encoded::<types::Array>(MapRef(&lines, String::as_str), b"\x92\xa1a\xa2bc");
    assert_encoded::<types::Object>(MapRef(&lines, String::as_str), b"\x92\xa1a\xa2bc");
    assert_eq!(lines, ["a", "bc"]);
}

#[test]
fn borrowed_mapping_preserves_exact_lengths_and_is_lazy() {
    let lines = [String::from("a"), String::from("bc")];
    let count = Cell::new(0);
    let encoded =
        <_ as Like<types::ArrayOf<types::Int>>>::encode(MapRef(&lines[..], |s: &String| {
            count.set(count.get() + 1);
            s.len() as i64
        }));
    let mut it = encoded.0.into_iter();
    assert_eq!(it.len(), 2);
    assert_eq!(it.size_hint(), (2, Some(2)));
    assert_eq!(count.get(), 0);
    assert_eq!(it.next(), Some(1));
    assert_eq!(it.len(), 1);
    assert_eq!(it.size_hint(), (1, Some(1)));
    assert_eq!(count.get(), 1);
    assert_eq!(it.next(), Some(2));
    assert_eq!(it.next(), None);
    assert_eq!(it.len(), 0);
    assert_eq!(it.size_hint(), (0, Some(0)));
    assert_eq!(count.get(), 2);
}

#[test]
fn mapping_keeps_array_lengths_available_to_fixed_length_markers() {
    assert_encoded::<types::FixedArrayOf<types::String, 2>>(
        Map([String::from("a"), String::from("bc")], Line),
        b"\x92\xa1a\xa2bc",
    );
    let lines = [String::from("a"), String::from("bc")];
    assert_encoded::<types::FixedArrayOf<types::String, 2>>(
        Map(&lines, String::as_str),
        b"\x92\xa1a\xa2bc",
    );
    assert_encoded::<types::FixedArrayOf<types::String, 2>>(
        MapRef(&lines, String::as_str),
        b"\x92\xa1a\xa2bc",
    );
}

#[test]
fn mapping_accepts_array_wrappers_without_losing_ownership_or_length() {
    assert_encoded::<types::FixedArrayOf<types::String, 2>>(
        Map(Array([Line("a".into()), Line("bc".into())]), |Line(s)| s),
        b"\x92\xa1a\xa2bc",
    );
    let lines = Array([String::from("a"), String::from("bc")]);
    assert_encoded::<types::FixedArrayOf<types::String, 2>>(
        Map(Array(&lines.0), String::as_str),
        b"\x92\xa1a\xa2bc",
    );
    assert_encoded::<types::FixedArrayOf<types::String, 2>>(
        MapRef(&lines, String::as_str),
        b"\x92\xa1a\xa2bc",
    );
    assert_encoded::<types::ArrayOf<types::String>>(
        Map(ArrayIter(lines.0.iter()), String::as_str),
        b"\x92\xa1a\xa2bc",
    );
}

#[test]
fn empty_mapping_writes_an_empty_array_without_calling_the_callback() {
    fn unreachable(_: &String) -> &str {
        panic!("empty mapping must not call its callback");
    }

    let lines: [String; 0] = [];
    assert_encoded::<types::ArrayOf<types::String>>(Map(&lines[..], unreachable), b"\x90");
    assert_encoded::<types::Array>(Map(&lines, unreachable), b"\x90");
    assert_encoded::<types::Object>(MapRef(&lines[..], unreachable), b"\x90");
    assert_encoded::<types::FixedArrayOf<types::String, 0>>(MapRef(&lines, unreachable), b"\x90");
    assert_encoded::<types::FixedArrayOf<types::String, 0>>(Map([] as [String; 0], Line), b"\x90");
}

#[test]
fn mapped_values_work_in_generated_typed_calls() {
    let lines = [String::from("a"), String::from("bc")];
    let mut nvim = Neovim::new(Vec::new());
    nvim.notify::<api::NvimBufSetLines>((
        Buffer::CURRENT,
        0,
        -1,
        true,
        MapRef(&lines[..], String::as_str),
    ))
    .unwrap();
    nvim.notify::<api::NvimWinSetCursor>((Window::CURRENT, Map([1i64, 2], |n| n + 1)))
        .unwrap();
    assert_eq!(
        nvim.into_inner(),
        b"\x93\x02\xb2nvim_buf_set_lines\x95\x00\x00\xff\xc3\x92\xa1a\xa2bc\x93\x02\xb3nvim_win_set_cursor\x92\x00\x92\x02\x03",
    );
    assert_eq!(lines, ["a", "bc"]);
}
