use nvim_rpc::api_types::encode::{Convert, Like, Untagged};
use nvim_rpc::mp::encode::primitive::{Array, ArrayIter, Map, MapIter};
use nvim_rpc::mp::encode::{StdWrite, WriterSyncExt as _};
use nvim_rpc::read::StdRead;
use nvim_rpc::rpc::{Message, MessageId};
use nvim_rpc::{Buffer, Neovim, Pending, Receiver, api, sys};

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::collections::BTreeMap;
use std::io::Cursor;

thread_local! {
    static ALLOCATIONS: Cell<Option<usize>> = const { Cell::new(None) };
}

struct Allocator;

fn record_allocation() {
    let _ = ALLOCATIONS.try_with(|count| {
        if let Some(n) = count.get() {
            count.set(Some(n + 1));
        }
    });
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record_allocation();
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record_allocation();
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        record_allocation();
        unsafe { System.realloc(ptr, layout, size) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

fn allocations(f: impl FnOnce()) -> usize {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            ALLOCATIONS.with(|count| count.set(None));
        }
    }

    ALLOCATIONS.with(|count| count.set(Some(0)));
    let _reset = Reset;
    f();
    ALLOCATIONS.with(|count| count.get().unwrap())
}

#[test]
fn borrowed_calls_iterators_and_responses_need_no_heap_storage() {
    let keys = BTreeMap::from([("alpha".to_owned(), ()), ("beta".to_owned(), ())]);
    let lines = ["first", "second"];
    let mut output = [0; 512];
    let mut cursor = Cursor::new(&mut output[..]);
    let mut nvim = Neovim::new(StdWrite::new(&mut cursor));
    let response = &[0x94, 1, 7, 0xc0, 0xa5, b'h', b'e', b'l', b'l', b'o'][..];

    let count = allocations(|| {
        nvim.request::<api::NvimBufSetLines>((Buffer(0), 0, -1, true, &lines[..]))
            .unwrap();
        nvim.notify::<api::NvimBufSetLines>((
            Buffer(0),
            0,
            -1,
            true,
            ArrayIter(keys.keys().map(String::as_str)),
        ))
        .unwrap();
        nvim.send(
            nvim_rpc::mp::encode::rpc::Response {
                msgid: MessageId(8),
                result: Ok::<_, &str>(42u8),
            }
            .to_primitive(),
        )
        .unwrap();

        let mut receiver = Receiver::new(StdRead(response), [0; 32]);
        let Message::Response(response) = receiver.recv().unwrap() else {
            panic!("expected response");
        };
        let text: &str = Pending::<sys::functions::NvimGetCurrentLine>::new(MessageId(7))
            .decode(response)
            .unwrap();
        assert_eq!(text, "hello");
    });
    assert_eq!(count, 0);
}

#[test]
fn collection_like_conversions_need_no_heap_storage() {
    struct Text(String);

    impl<'s> Like<sys::types::String> for &'s Text {
        type EncodeAs = &'s str;

        fn encode(self) -> Self::EncodeAs {
            &self.0
        }
    }

    let lines = [Text("first".into()), Text("second".into())];
    let key = Text("buffer".into());
    let buffers = [Buffer(7), Buffer(9)];
    let mut output = [0; 1024];
    let mut nvim = Neovim::new(StdWrite::new(Cursor::new(&mut output[..])));

    let count = allocations(|| {
        nvim.notify::<api::NvimBufSetLines>((Buffer(0), 0, -1, true, &lines[..]))
            .unwrap();
        nvim.notify::<api::NvimBufSetLines>((Buffer(0), 0, -1, true, Convert(Array(&lines))))
            .unwrap();
        nvim.notify::<api::NvimBufSetLines>((
            Buffer(0),
            0,
            -1,
            true,
            Convert(ArrayIter(lines.iter())),
        ))
        .unwrap();
        nvim.notify::<api::NvimBufSetLines>((
            Buffer(0),
            0,
            -1,
            true,
            Convert((&lines[0], &lines[1])),
        ))
        .unwrap();
        nvim.notify::<api::NvimSetVar>(("buffers", Convert(Array(&buffers))))
            .unwrap();
        nvim.request::<api::NvimGetOptionValue>(("number", Convert(Map([(&key, &buffers[0])]))))
            .unwrap();
        nvim.request::<api::NvimGetOptionValue>((
            "number",
            Convert(MapIter([(&key, &buffers[1])].into_iter())),
        ))
        .unwrap();
    });
    assert_eq!(count, 0);
}

#[test]
fn mapped_calls_need_no_heap_storage() {
    use nvim_rpc::api_types::encode::{Map, MapRef};

    let lines = [String::from("first"), String::from("second")];
    let owned = [String::from("third"), String::from("fourth")];
    let mut output = [0; 512];
    let mut nvim = Neovim::new(StdWrite::new(Cursor::new(&mut output[..])));

    let count = allocations(|| {
        nvim.notify::<api::NvimBufSetLines>((
            Buffer::CURRENT,
            0,
            -1,
            true,
            MapRef(&lines, String::as_str),
        ))
        .unwrap();
        nvim.notify::<api::NvimBufSetLines>((
            Buffer::CURRENT,
            0,
            -1,
            true,
            Map(owned, String::into_bytes),
        ))
        .unwrap();
        nvim.notify::<api::NvimSetVar>(("buffers", Map([7, 9], Buffer::new)))
            .unwrap();
    });
    assert_eq!(count, 0);
    assert_eq!(lines, ["first", "second"]);
}

#[test]
fn untagged_arguments_and_collections_need_no_heap_storage() {
    use sys::types::{ArrayOf, OneOf};

    enum Value {
        Int(i64),
        Text(String),
    }

    impl<'s> Like<OneOf<sys::types::Int, sys::types::String>> for &'s Value {
        type EncodeAs = Untagged<i64, &'s str>;

        fn encode(self) -> Self::EncodeAs {
            match self {
                Value::Int(n) => Untagged::Left(*n),
                Value::Text(s) => Untagged::Right(s),
            }
        }
    }

    type Marker = OneOf<sys::types::Int, sys::types::String>;
    let values = [Value::Int(7), Value::Text("text".into())];
    let mut output = [0; 128];
    let mut cursor = Cursor::new(&mut output[..]);
    let mut writer = StdWrite::new(&mut cursor);

    let count = allocations(|| {
        for value in &values {
            writer
                .write_value(<_ as Like<Marker>>::encode(value))
                .consume()
                .unwrap();
        }
        writer
            .write_primitive(<_ as Like<ArrayOf<Marker>>>::encode(&values[..]))
            .consume()
            .unwrap();
        writer
            .write_primitive(Map([("value", <_ as Like<Marker>>::encode(&values[1]))]))
            .consume()
            .unwrap();
    });
    assert_eq!(count, 0);
    let len = cursor.position() as usize;
    assert_eq!(
        &cursor.get_ref()[..len],
        b"\x07\xa4text\x92\x07\xa4text\x81\xa5value\xa4text"
    );
}
