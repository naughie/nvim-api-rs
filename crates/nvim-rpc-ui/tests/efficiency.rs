use nvim_rpc::mp::encode::{Primitive, WriterSyncExt as _};
use nvim_rpc::rpc::Message;
use nvim_rpc_ui::{Event, Frame, GridId, Model};

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::Arc;

thread_local! {
    static ALLOCATIONS: Cell<Option<usize>> = const { Cell::new(None) };
}

struct Allocator;

fn record_allocation() {
    let _ = ALLOCATIONS.try_with(|count| {
        if let Some(n) = count.get() {
            count.set(Some(n.saturating_add(1)));
        }
    });
}

// This allocator adds no allocation metadata and returns System's pointers
// unchanged. Consequently every live allocation has exactly System's layout
// and ownership, including after realloc. record_allocation uses only a Cell,
// does not allocate, and cannot unwind (try_with handles TLS destruction and
// the counter saturates). Thus forwarding also satisfies GlobalAlloc's
// prohibition on unwinding and recursive allocation.
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record_allocation();
        // GlobalAlloc's caller supplies a valid nonzero layout; it is unchanged.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record_allocation();
        // The same valid layout is forwarded, retaining System's zeroing.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        record_allocation();
        // All pointers originate unchanged from System. The caller guarantees
        // ptr is live with layout and that size is nonzero and fits isize.
        unsafe { System.realloc(ptr, layout, size) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // The caller supplies a live pointer and its original layout; both
        // were issued unchanged by System, so its deallocation contract holds.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

fn allocations<T>(f: impl FnOnce() -> T) -> (T, usize) {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            ALLOCATIONS.with(|count| count.set(None));
        }
    }

    ALLOCATIONS.with(|count| count.set(Some(0)));
    let _reset = Reset;
    let ret = f();
    let count = ALLOCATIONS.with(|count| count.get().unwrap());
    (ret, count)
}

fn redraw(params: impl Primitive) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.write_primitive((2u8, "redraw", params)).consume();
    bytes
}

fn apply(model: &mut Model, bytes: &[u8], emit: impl FnMut(Frame)) {
    let Message::Notification(notification) = nvim_rpc::rpc::decode(&mut &bytes[..]).unwrap()
    else {
        panic!("expected redraw notification");
    };
    model.apply_redraw(notification, emit).unwrap();
}

fn frame(model: &mut Model, bytes: &[u8]) -> Frame {
    let mut ret = None;
    apply(model, bytes, |frame| {
        assert!(ret.is_none(), "expected exactly one flush");
        ret = Some(frame);
    });
    ret.expect("expected a flush")
}

#[test]
fn cursor_only_coalescing_does_not_allocate_or_copy_grids() {
    let mut model = Model::default();
    let initial = frame(
        &mut model,
        &redraw((("grid_resize", (1, 120, 40)), ("flush", ()))),
    );
    let mut pending = frame(
        &mut model,
        &redraw((("grid_cursor_goto", (1, 10, 20)), ("flush", ()))),
    );
    let next = frame(
        &mut model,
        &redraw((("grid_cursor_goto", (1, 10, 21)), ("flush", ()))),
    );
    let latest = Arc::clone(&next.state);
    let (result, count) = allocations(|| pending.coalesce_incremental(next, 0, 1));
    result.unwrap();
    assert_eq!(count, 0);
    assert!(Arc::ptr_eq(&pending.state, &latest));
    assert!(Arc::ptr_eq(&pending.state.grids, &initial.state.grids));
    assert!(pending.damage.cursor);
    assert!(!pending.damage.full);
    assert!(pending.damage.grids.is_empty());
}

#[test]
fn empty_flushes_reuse_published_state_without_allocating() {
    let mut model = Model::default();
    let initial = frame(
        &mut model,
        &redraw((("grid_resize", (1, 120, 40)), ("flush", ()))),
    );
    let bytes = redraw((("flush", ()),));

    let (_, count) = allocations(|| {
        for sequence in 2..=65 {
            let next = frame(&mut model, &bytes);
            assert!(Arc::ptr_eq(&initial.state, &next.state));
            assert_eq!(next.sequence, sequence);
            assert!(next.damage.grids.is_empty());
            assert!(next.events.is_empty());
        }
    });
    assert_eq!(count, 0);
}

#[test]
fn event_only_flushes_reuse_state_and_preserve_order() {
    let mut model = Model::default();
    let initial = frame(
        &mut model,
        &redraw((("grid_resize", (1, 4, 2)), ("flush", ()))),
    );
    let next = frame(
        &mut model,
        &redraw((("bell", ()), ("visual_bell", ()), ("flush", ()))),
    );
    assert!(Arc::ptr_eq(&initial.state, &next.state));
    assert!(matches!(
        next.events.as_slice(),
        [Event::Bell, Event::VisualBell]
    ));
    assert!(next.damage.grids.is_empty());
}

#[test]
fn cursor_updates_keep_all_grid_storage_shared() {
    let mut model = Model::default();
    let initial = frame(
        &mut model,
        &redraw((("grid_resize", (1, 120, 40), (2, 20, 10)), ("flush", ()))),
    );
    let next = frame(
        &mut model,
        &redraw((("grid_cursor_goto", (1, 39, 119)), ("flush", ()))),
    );

    assert!(initial.state.cursor.is_none());
    let cursor = next.state.cursor.unwrap();
    assert_eq!((cursor.grid, cursor.row, cursor.col), (GridId(1), 39, 119));
    assert!(Arc::ptr_eq(&initial.state.grids, &next.state.grids));
    assert!(Arc::ptr_eq(
        &initial.state.placements,
        &next.state.placements
    ));
    assert!(Arc::ptr_eq(&initial.state.viewports, &next.state.viewports));
    assert!(Arc::ptr_eq(&initial.state.cmdlines, &next.state.cmdlines));
    for id in [GridId(1), GridId(2)] {
        assert!(Arc::ptr_eq(
            &initial.state.grids[&id],
            &next.state.grids[&id]
        ));
    }
    assert!(next.damage.cursor);
    assert!(next.damage.grids.is_empty());
}

#[test]
fn whole_row_scrolls_share_rows_and_leave_retained_frames_stable() {
    for rows in [-3i64, -1, 0, 1, 3] {
        let mut model = Model::default();
        let initial = frame(
            &mut model,
            &redraw((
                ("grid_resize", (1, 3, 4)),
                (
                    "grid_line",
                    (1, 0, 0, (("a", 0, 3),), false),
                    (1, 1, 0, (("b", 1, 3),), true),
                    (1, 2, 0, (("c", 2, 3),), false),
                    (1, 3, 0, (("d", 3, 3),), true),
                ),
                ("flush", ()),
            )),
        );
        let next = frame(
            &mut model,
            &redraw((("grid_scroll", (1, 0, 4, 0, 3, rows, 0)), ("flush", ()))),
        );

        let before = &initial.state.grids[&GridId(1)];
        let after = &next.state.grids[&GridId(1)];
        for row in 0..4 {
            let src = row as i64 + rows;
            let src = if (0..4).contains(&src) {
                src as usize
            } else {
                row
            };
            assert!(
                Arc::ptr_eq(&before.rows[src], &after.rows[row]),
                "row {row}, scroll {rows}"
            );
            assert_eq!(
                before.cell(row, 0).unwrap().text.as_ref(),
                ["a", "b", "c", "d"][row]
            );
        }
    }
}

#[test]
fn unpublished_whole_row_scrolls_need_no_scratch_allocation() {
    let mut model = Model::default();
    let initial = redraw((("grid_resize", (1, 120, 40)),));
    apply(&mut model, &initial, |_| panic!("unexpected flush"));
    let bytes = redraw((("grid_scroll", (1, 0, 40, 0, 120, 1, 0)),));

    let (_, count) = allocations(|| {
        apply(&mut model, &bytes, |_| panic!("unexpected flush"));
    });
    assert_eq!(count, 0);
}
