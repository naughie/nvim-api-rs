use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    static ALLOCATIONS: Cell<Option<usize>> = const { Cell::new(None) };
}

pub(super) struct Allocator;

fn record_allocation() {
    let _ = ALLOCATIONS.try_with(|count| {
        if let Some(n) = count.get() {
            count.set(Some(n.saturating_add(1)));
        }
    });
}

// Pointers, layouts, and ownership pass through unchanged to System. The
// counter uses only a Cell, never allocates, and cannot unwind: try_with handles
// TLS destruction and addition saturates. Forwarding therefore also preserves
// GlobalAlloc's non-recursion and non-unwinding requirements.
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record_allocation();
        // The caller provides a valid nonzero layout, which is unchanged.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record_allocation();
        // The same valid layout reaches System, preserving its zeroing.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        record_allocation();
        // System issued ptr unchanged; the caller guarantees its live layout
        // and that size is nonzero and fits isize.
        unsafe { System.realloc(ptr, layout, size) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // System issued this live pointer with the same layout and ownership.
        unsafe { System.dealloc(ptr, layout) }
    }
}

pub(super) fn count<T>(f: impl FnOnce() -> T) -> (T, usize) {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            ALLOCATIONS.with(|count| count.set(None));
        }
    }

    ALLOCATIONS.with(|count| count.set(Some(0)));
    let _reset = Reset;
    let ret = f();
    let count = ALLOCATIONS.with(|count| count.get().unwrap_or(0));
    (ret, count)
}
