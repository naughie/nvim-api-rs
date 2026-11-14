use nvim_api::mp::encode::StdWrite;
use nvim_api::mp::encode::primitive::ArrayIter;
use nvim_api::read::StdRead;
use nvim_api::rpc::{Message, MessageId};
use nvim_api::{Buffer, Neovim, Pending, Receiver, api, sys};

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
            nvim_api::mp::encode::rpc::Response {
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
