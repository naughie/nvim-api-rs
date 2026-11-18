//! Measures redraw allocations, processing, and backend presentation while
//! retaining the previous frame. Run with `cargo run --release -p nvim-rpc-ui --example replay
//! -- 100000`. Timings are workload measurements, not a performance guarantee.
use nvim_rpc::mp::encode::primitive::ArrayIter;
use nvim_rpc::mp::encode::{Primitive, WriterSyncExt as _};
use nvim_rpc::rpc::Message;
use nvim_rpc_ui::{Backend, Frame, Ui};

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::convert::Infallible;
use std::error::Error;
use std::hint::black_box;
use std::time::Instant;

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
    let count = ALLOCATIONS.with(|count| count.get().unwrap_or(0));
    (ret, count)
}

fn encode(params: impl Primitive) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.write_primitive((2u8, "redraw", params)).consume();
    bytes
}

struct Recorder {
    frame: Option<Frame>,
}

impl Backend for Recorder {
    type Error = Infallible;

    fn present(&mut self, frame: Frame) -> Result<(), Self::Error> {
        self.frame = Some(black_box(frame));
        Ok(())
    }
}

fn apply(ui: &mut Ui<Recorder>, mut bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let Message::Notification(notification) = nvim_rpc::rpc::decode(&mut bytes)? else {
        return Err("expected a redraw notification".into());
    };
    ui.redraw(notification)?;
    Ok(())
}

fn measure(name: &str, frames: &[Vec<u8>], iterations: u64) -> Result<(), Box<dyn Error>> {
    if frames.is_empty() {
        return Err("workload must contain a frame".into());
    }
    let mut ui = Ui::new(Recorder { frame: None });
    apply(
        &mut ui,
        &encode((("grid_resize", (1, 120, 40)), ("flush", ()))),
    )?;
    let mut frames = frames.iter().cycle();
    for bytes in frames.by_ref().take(1000) {
        apply(&mut ui, bytes)?;
    }
    let start = Instant::now();
    for _ in 0..iterations {
        let bytes = frames.next().ok_or("workload ended unexpectedly")?;
        apply(&mut ui, black_box(bytes))?;
    }
    let elapsed = start.elapsed();
    let samples = iterations.min(1000);
    let (ret, count) = allocations(|| -> Result<(), Box<dyn Error>> {
        for _ in 0..samples {
            let bytes = frames.next().ok_or("workload ended unexpectedly")?;
            apply(&mut ui, bytes)?;
        }
        Ok(())
    });
    ret?;
    let frame = ui
        .into_backend()
        .frame
        .ok_or("fixture did not publish a frame")?;
    let expected = iterations
        .checked_add(1001)
        .and_then(|n| n.checked_add(samples))
        .ok_or("frame count overflow")?;
    if frame.sequence != expected {
        return Err("fixture published an unexpected frame count".into());
    }
    println!(
        "{name}: {:.0} ns/frame; {:.0} frames/s; {:.2} allocations/frame",
        elapsed.as_nanos() as f64 / iterations as f64,
        iterations as f64 / elapsed.as_secs_f64(),
        count as f64 / samples as f64,
    );
    Ok(())
}

fn line<'a>(text: impl ExactSizeIterator<Item = &'a str>) -> Vec<u8> {
    encode((
        (
            "grid_line",
            (1, 10, 0, ArrayIter(text.map(|s| (s, 0))), false),
        ),
        ("flush", ()),
    ))
}

fn repeated(text: &[&str]) -> Vec<u8> {
    let row = text.iter().copied().cycle().take(120).collect::<Vec<_>>();
    line(row.into_iter())
}

fn vocabulary(count: usize, recurring: bool) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
    // This CJK range contains 15,360 valid scalars. Frames are encoded before
    // measurement; padding the final row keeps every update at 120 entries.
    // The fixed chunk size 120 is nonzero.
    let text = (0x4e00..0x8a00)
        .take(count)
        .map(|n| char::from_u32(n).map(String::from).ok_or("invalid scalar"))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(text
        .chunks(120)
        .map(|row| {
            let row = row
                .iter()
                .chain(text.iter())
                .take(120)
                .enumerate()
                .map(|(n, s)| {
                    // The fixed divisor 8 is nonzero.
                    if recurring && n % 8 == 0 {
                        "日"
                    } else {
                        s.as_str()
                    }
                })
                .collect::<Vec<_>>();
            line(row.into_iter())
        })
        .collect())
}

fn main() -> Result<(), Box<dyn Error>> {
    let iterations = std::env::args()
        .nth(1)
        .map(|v| v.parse::<u64>())
        .transpose()?
        .unwrap_or(100_000);
    if iterations == 0 || iterations.checked_add(2001).is_none() {
        return Err("iterations must be between 1 and u64::MAX - 2001".into());
    }
    let typing = encode((
        ("grid_line", (1, 10, 20, (("x", 0),), false)),
        ("grid_cursor_goto", (1, 10, 21)),
        ("flush", ()),
    ));
    let scroll = encode((
        ("grid_scroll", (1, 0, 40, 0, 120, 1, 0)),
        ("grid_line", (1, 39, 0, ((" ", 0, 120),), false)),
        ("flush", ()),
    ));
    measure("typing 120x40", &[typing], iterations)?;
    measure("scroll 120x40", &[scroll], iterations)?;
    for (name, text) in [
        ("ASCII row", &["a", "b", "c", "d"][..]),
        ("CJK row", &["日", "本", "語", "界"][..]),
        (
            "combining row",
            &["e\u{301}", "a\u{308}", "o\u{302}", "n\u{303}"][..],
        ),
        ("emoji row", &["😀", "🚀", "👩‍💻", "🇯🇵"][..]),
        (
            "multi-character row",
            &["hello", "world", "日本語", "<tab>"][..],
        ),
    ] {
        measure(name, &[repeated(text)], iterations)?;
    }
    for (name, count) in [
        ("256-scalar CJK working set", 256),
        ("1024-scalar CJK working set", 1024),
        ("4096-scalar CJK working set", 4096),
    ] {
        measure(name, &vocabulary(count, false)?, iterations)?;
    }
    measure(
        "mostly unique CJK rows",
        &vocabulary(15_360, true)?,
        iterations,
    )?;
    measure(
        "cache-churn CJK rows",
        &vocabulary(15_360, false)?,
        iterations,
    )
}
