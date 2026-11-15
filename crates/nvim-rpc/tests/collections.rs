use nvim_rpc::api_types::BaseType;
use nvim_rpc::api_types::encode::{Convert, Like};
use nvim_rpc::mp::encode::WriterSyncExt as _;
use nvim_rpc::mp::encode::primitive::{Array, ArrayIter, Map, MapIter, Primitive, State as _};
use nvim_rpc::sys::{ext_types as ext, types};
use nvim_rpc::{Buffer, Neovim, Tabpage, Window, api};

use std::cell::Cell;
use std::collections::BTreeMap;

struct Line(String);

impl Like<types::String> for Line {
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

impl Like<types::Object> for Line {
    type EncodeAs = String;

    fn encode(self) -> Self::EncodeAs {
        self.0
    }
}

impl<'s> Like<types::Object> for &'s Line {
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
fn borrowed_custom_strings_compose_in_slices_and_arrays() {
    let lines = [Line("a".into()), Line("bc".into())];
    let expected = b"\x92\xa1a\xa2bc";

    assert_encoded::<types::ArrayOf<types::String>>(&lines[..], expected);
    assert_encoded::<types::ArrayOf<types::String>>(&lines, expected);
    assert_encoded::<types::FixedArrayOf<types::String, 2>>(&lines, expected);
    assert_encoded::<types::ArrayOf<types::String>>(Convert(Array(&lines)), expected);
    assert_encoded::<types::FixedArrayOf<types::String, 2>>(Convert(Array(&lines)), expected);
    assert_encoded::<types::ArrayOf<types::String>>(Convert(ArrayIter(lines.iter())), expected);
    assert_eq!(lines[0].0, "a");
    assert_eq!(lines[1].0, "bc");
}

#[test]
fn owned_custom_values_move_into_collection_encodings() {
    assert_encoded::<types::ArrayOf<types::String>>(
        Convert(Array([Line("a".into()), Line("bc".into())])),
        b"\x92\xa1a\xa2bc",
    );
    assert_encoded::<types::FixedArrayOf<types::String, 2>>(
        Convert(Array([Line("a".into()), Line("bc".into())])),
        b"\x92\xa1a\xa2bc",
    );
    assert_encoded::<types::ArrayOf<types::String>>(
        Convert(ArrayIter([Line("a".into()), Line("bc".into())].into_iter())),
        b"\x92\xa1a\xa2bc",
    );
    assert_encoded::<types::Dict>(
        Convert(Map([(Line("key".into()), Line("value".into()))])),
        b"\x81\xa3key\xa5value",
    );
}

#[test]
fn handle_collections_apply_owned_and_borrowed_conversions() {
    fn check<M: BaseType, T: Copy + Like<M>>(handles: [T; 2])
    where
        for<'s> &'s T: Like<M>,
    {
        assert_encoded::<types::ArrayOf<M>>(&handles[..], b"\x92\x07\x09");
        assert_encoded::<types::FixedArrayOf<M, 2>>(&handles, b"\x92\x07\x09");
        assert_encoded::<types::ArrayOf<M>>(Convert(Array(handles)), b"\x92\x07\x09");
        assert_encoded::<types::FixedArrayOf<M, 2>>(Convert(Array(handles)), b"\x92\x07\x09");
    }

    check::<ext::Buffer, Buffer>([Buffer::new(7), Buffer::new(9)]);
    check::<ext::Window, Window>([Window::new(7), Window::new(9)]);
    check::<ext::Tabpage, Tabpage>([Tabpage::new(7), Tabpage::new(9)]);
}

#[test]
fn empty_collections_preserve_their_wire_kind() {
    let lines: [Line; 0] = [];
    assert_encoded::<types::ArrayOf<types::String>>(&lines[..], b"\x90");
    assert_encoded::<types::ArrayOf<types::String>>(&lines, b"\x90");
    assert_encoded::<types::FixedArrayOf<types::String, 0>>(&lines, b"\x90");
    assert_encoded::<types::ArrayOf<types::String>>(Convert(Array(&lines)), b"\x90");
    assert_encoded::<types::FixedArrayOf<types::String, 0>>(Convert(Array(&lines)), b"\x90");
    assert_encoded::<types::ArrayOf<types::String>>(Convert(ArrayIter(lines.iter())), b"\x90");
    assert_encoded::<types::ArrayOf<types::String>>(Convert(ArrayIter(lines.into_iter())), b"\x90");
    assert_encoded::<types::Array>(Convert(()), b"\x90");
    assert_encoded::<types::ArrayOf<types::String>>(Convert(()), b"\x90");
    assert_encoded::<types::FixedArrayOf<types::String, 0>>(Convert(()), b"\x90");

    assert_encoded::<types::Dict>(Convert(Map::<[(Line, Buffer); 0]>([])), b"\x80");
    assert_encoded::<types::Dict>(
        Convert(MapIter(([] as [(Line, Buffer); 0]).into_iter())),
        b"\x80",
    );
    assert_encoded::<types::Object>(Convert(Map::<[(Buffer, Window); 0]>([])), b"\x80");
}

#[test]
fn tuples_convert_heterogeneous_elements_under_the_selected_marker() {
    let line = Line("a".into());
    assert_encoded::<types::ArrayOf<types::String>>(
        Convert((&line, "b", String::from("c"))),
        b"\x93\xa1a\xa1b\xa1c",
    );
    assert_encoded::<types::FixedArrayOf<types::String, 3>>(
        Convert((&line, "b", Line("c".into()))),
        b"\x93\xa1a\xa1b\xa1c",
    );
    assert_encoded::<types::Array>(
        Convert((
            Buffer::new(7),
            &Window::new(9),
            Tabpage::new(11),
            true,
            &line,
        )),
        b"\x95\x07\x09\x0b\xc3\xa1a",
    );
    assert_encoded::<types::Object>(
        Convert((Buffer::new(7), &Window::new(9), Tabpage::new(11))),
        b"\x93\x07\x09\x0b",
    );
    assert_encoded::<types::FixedArrayOf<ext::Buffer, 1>>(Convert((Buffer::new(7),)), b"\x91\x07");
    assert_encoded::<types::FixedArrayOf<ext::Buffer, 15>>(
        Convert((
            Buffer(1),
            Buffer(2),
            Buffer(3),
            Buffer(4),
            Buffer(5),
            Buffer(6),
            Buffer(7),
            Buffer(8),
            Buffer(9),
            Buffer(10),
            Buffer(11),
            Buffer(12),
            Buffer(13),
            Buffer(14),
            Buffer(15),
        )),
        b"\x9f\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f",
    );
}

#[test]
fn untyped_arrays_convert_elements_as_objects() {
    let handles = [Buffer::new(7), Buffer::new(9)];
    assert_encoded::<types::Array>(Convert(Array(&handles)), b"\x92\x07\x09");
    assert_encoded::<types::Object>(Convert(Array(&handles)), b"\x92\x07\x09");
    assert_encoded::<types::Array>(Convert(ArrayIter(handles.iter())), b"\x92\x07\x09");
    assert_encoded::<types::Object>(Convert(ArrayIter(handles.into_iter())), b"\x92\x07\x09");
}

#[test]
fn map_iterators_convert_borrowed_keys_and_values() {
    let entries = BTreeMap::from([("a".to_owned(), Buffer(7)), ("b".to_owned(), Buffer(9))]);
    let expected = b"\x82\xa1a\x07\xa1b\x09";
    assert_encoded::<types::Dict>(Convert(Map(&entries)), expected);
    assert_encoded::<types::Dict>(Convert(MapIter(entries.iter())), expected);

    let key = Line("key".into());
    let value = Line("value".into());
    assert_encoded::<types::Dict>(Convert(Map([(&key, &value)])), b"\x81\xa3key\xa5value");
    assert_encoded::<types::Dict>(
        Convert(MapIter([(&key, &value)].into_iter())),
        b"\x81\xa3key\xa5value",
    );
    assert_encoded::<types::Object>(Convert(Map([(Buffer(7), Window(9))])), b"\x81\x07\x09");
    assert_encoded::<types::Object>(
        Convert(MapIter([(&Buffer(7), &Window(9))].into_iter())),
        b"\x81\x07\x09",
    );
}

#[test]
fn nested_collection_conversions_compose() {
    let lines = [Line("a".into()), Line("b".into())];
    assert_encoded::<types::ArrayOf<types::ArrayOf<types::String>>>(
        Convert(Array([&lines[..1], &lines[1..]])),
        b"\x92\x91\xa1a\x91\xa1b",
    );
    assert_encoded::<types::Dict>(
        Convert(Map([(
            Line("buffers".into()),
            Convert(Array([Buffer(7), Buffer(9)])),
        )])),
        b"\x81\xa7buffers\x92\x07\x09",
    );
    assert_encoded::<types::ArrayOf<types::Dict>>(
        Convert(Array([
            Convert(Map([(Line("a".into()), Buffer(7))])),
            Convert(Map([(Line("b".into()), Buffer(9))])),
        ])),
        b"\x92\x81\xa1a\x07\x81\xa1b\x09",
    );
}

struct Counted<'s>(&'s Cell<usize>, &'s str);

impl<'s> Like<types::String> for Counted<'s> {
    type EncodeAs = &'s str;

    fn encode(self) -> Self::EncodeAs {
        self.0.set(self.0.get() + 1);
        self.1
    }
}

impl<'s> Like<types::Object> for Counted<'s> {
    type EncodeAs = &'s str;

    fn encode(self) -> Self::EncodeAs {
        <Self as Like<types::String>>::encode(self)
    }
}

#[test]
fn array_conversion_preserves_length_and_runs_only_as_elements_are_read() {
    let count = Cell::new(0);
    let encoded = <_ as Like<types::ArrayOf<types::String>>>::encode(Convert(Array([
        Counted(&count, "a"),
        Counted(&count, "b"),
    ])));
    let mut it = encoded.0.into_iter();
    assert_eq!(it.len(), 2);
    assert_eq!(it.size_hint(), (2, Some(2)));
    assert_eq!(count.get(), 0);
    assert_eq!(it.next(), Some("a"));
    assert_eq!(count.get(), 1);
    assert_eq!(it.len(), 1);
    assert_eq!(it.size_hint(), (1, Some(1)));
    assert_eq!(it.next(), Some("b"));
    assert_eq!(it.next(), None);
    assert_eq!(it.len(), 0);
    assert_eq!(count.get(), 2);

    count.set(0);
    let encoded = <_ as Like<types::ArrayOf<types::String>>>::encode(Convert(ArrayIter(
        [Counted(&count, "a"), Counted(&count, "b")].into_iter(),
    )));
    let mut state = encoded.into_state();
    assert_eq!(count.get(), 0);
    assert_eq!(state.next().unwrap().as_ref(), b"\x92");
    assert_eq!(count.get(), 0);
    assert_eq!(state.next().unwrap().as_ref(), b"\xa1");
    assert_eq!(count.get(), 1);
    assert_eq!(state.next().unwrap().as_ref(), b"a");
    assert_eq!(count.get(), 1);
    assert_eq!(state.next().unwrap().as_ref(), b"\xa1");
    assert_eq!(count.get(), 2);
    assert_eq!(state.next().unwrap().as_ref(), b"b");
    assert!(state.next().is_none());
    assert_eq!(count.get(), 2);
}

#[test]
fn array_iterators_encode_their_remaining_elements() {
    let mut values = [7i64, 9].into_iter();
    assert_eq!(values.next(), Some(7));
    assert_encoded::<types::ArrayOf<types::Int>>(ArrayIter(values), b"\x91\x09");

    let mut lines = [Line("a".into()), Line("bc".into())].into_iter();
    lines.next();
    assert_encoded::<types::ArrayOf<types::String>>(Convert(ArrayIter(lines)), b"\x91\xa2bc");
}

#[test]
fn map_conversion_preserves_pair_count_and_is_lazy() {
    let count = Cell::new(0);
    let encoded = <_ as Like<types::Dict>>::encode(Convert(Map([
        (Counted(&count, "a"), Counted(&count, "b")),
        (Counted(&count, "c"), Counted(&count, "d")),
    ])));
    let mut it = encoded.0.into_iter();
    assert_eq!(it.len(), 2);
    assert_eq!(it.size_hint(), (2, Some(2)));
    assert_eq!(count.get(), 0);
    assert_eq!(it.next(), Some(("a", "b")));
    assert_eq!(it.len(), 1);
    assert_eq!(it.size_hint(), (1, Some(1)));
    assert_eq!(count.get(), 2);
    assert_eq!(it.next(), Some(("c", "d")));
    assert_eq!(it.next(), None);
    assert_eq!(it.len(), 0);
    assert_eq!(count.get(), 4);

    count.set(0);
    let encoded = <_ as Like<types::Dict>>::encode(Convert(MapIter(
        [(Counted(&count, "a"), Counted(&count, "b"))].into_iter(),
    )));
    let mut state = encoded.into_state();
    assert_eq!(count.get(), 0);
    assert_eq!(state.next().unwrap().as_ref(), b"\x81");
    assert_eq!(count.get(), 0);
    assert_eq!(state.next().unwrap().as_ref(), b"\xa1");
    assert_eq!(count.get(), 2);
    assert_eq!(state.next().unwrap().as_ref(), b"a");
    assert_eq!(state.next().unwrap().as_ref(), b"\xa1");
    assert_eq!(state.next().unwrap().as_ref(), b"b");
    assert!(state.next().is_none());
    assert_eq!(count.get(), 2);
}

#[test]
fn converted_collections_work_in_generated_calls() {
    let lines = [Line("a".into()), Line("b".into())];
    let mut nvim = Neovim::new(Vec::new());
    nvim.notify::<api::NvimBufSetLines>((Buffer::CURRENT, 0, -1, true, &lines[..]))
        .unwrap();
    nvim.notify::<api::NvimGetOptionValue>((
        "number",
        Convert(Map([(Line("scope".into()), Line("local".into()))])),
    ))
    .unwrap();
    let bytes = nvim.into_inner();
    assert_eq!(
        bytes,
        b"\x93\x02\xb2nvim_buf_set_lines\x95\x00\x00\xff\xc3\x92\xa1a\xa1b\x93\x02\xb5nvim_get_option_value\x92\xa6number\x81\xa5scope\xa5local",
    );
}

#[test]
fn primitive_collection_arguments_keep_their_existing_encodings() {
    fn assert_traits<T: Copy + Clone + Eq>(_: &T) {}

    let values = ["a", "b"];
    let encoded = <_ as Like<types::ArrayOf<types::String>>>::encode(&values[..]);
    assert_traits(&encoded);
    let copied = encoded;
    assert!(encoded == copied);

    assert_encoded::<types::ArrayOf<types::String>>(Array(["a", "b"]), b"\x92\xa1a\xa1b");
    assert_encoded::<types::ArrayOf<types::String>>(
        ArrayIter(["a", "b"].into_iter()),
        b"\x92\xa1a\xa1b",
    );
    assert_encoded::<types::FixedArrayOf<types::Int, 2>>((7i64, 9i64), b"\x92\x07\x09");
    assert_encoded::<types::Dict>(Map([("a", 7u8)]), b"\x81\xa1a\x07");
    assert_encoded::<types::Dict>(MapIter([("a", 7u8)].into_iter()), b"\x81\xa1a\x07");
    assert_encoded::<types::Object>(Map([(7u8, 9u8)]), b"\x81\x07\x09");
}
