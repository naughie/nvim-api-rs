//! Measures redraw allocations, processing, and CPU cell painting, including
//! eight-frame bursts. Run with `cargo run --release -p nvim-rpc-ui --example
//! replay -- 100000`. Timings are workload measurements, not a performance
//! guarantee.
use nvim_rpc::mp::encode::primitive::ArrayIter;
use nvim_rpc::mp::encode::{Primitive, WriterSyncExt as _};
use nvim_rpc::rpc::Message;
use nvim_rpc_ui::{Backend, Frame, Grid, GridDamage, ProtocolError, Ui};

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::convert::Infallible;
use std::error::Error;
use std::hint::black_box;
use std::time::{Duration, Instant};

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

fn apply<B: Backend>(ui: &mut Ui<B>, mut bytes: &[u8]) -> Result<(), Box<dyn Error>>
where
    B::Error: Error + 'static,
{
    let Message::Notification(notification) = nvim_rpc::rpc::decode(&mut bytes)? else {
        return Err("expected a redraw notification".into());
    };
    ui.redraw(notification)?;
    Ok(())
}

#[derive(Default)]
struct Painter {
    cells: u128,
    cursors: u128,
    elapsed: Duration,
}

impl Painter {
    fn region(&mut self, grid: &Grid, region: GridDamage) {
        for row in grid
            .rows
            .iter()
            .skip(region.top)
            .take(region.bottom.saturating_sub(region.top))
        {
            for cell in row
                .cells
                .iter()
                .skip(region.left)
                .take(region.right.saturating_sub(region.left))
            {
                let hash = black_box(cell.text.as_bytes())
                    .iter()
                    .fold(black_box(cell.highlight.0 as u64), |hash, &byte| {
                        hash.rotate_left(5) ^ u64::from(byte)
                    });
                black_box(hash);
                self.cells = self.cells.saturating_add(1);
            }
        }
    }
}

impl Backend for Painter {
    type Error = Infallible;

    fn present(&mut self, frame: Frame) -> Result<(), Self::Error> {
        let start = Instant::now();
        if frame.damage.full {
            for (&id, grid) in frame.state.grids.iter() {
                self.region(
                    grid,
                    GridDamage {
                        grid: id,
                        top: 0,
                        bottom: grid.height,
                        left: 0,
                        right: grid.width,
                    },
                );
            }
        } else {
            for &region in &frame.damage.grids {
                if let Some(grid) = frame.state.grids.get(&region.grid) {
                    self.region(grid, region);
                }
            }
        }
        if frame.damage.full || frame.damage.cursor {
            black_box((&frame.state.cursor, &frame.state.mode, frame.state.busy));
            self.cursors = self.cursors.saturating_add(1);
        }
        for event in frame.events {
            black_box(event);
        }
        self.elapsed = self.elapsed.saturating_add(start.elapsed());
        Ok(())
    }
}

struct Burst {
    pending: Option<Frame>,
    frames: u8,
    incremental: bool,
    painter: Painter,
}

impl Burst {
    fn paint(&mut self) {
        if let Some(frame) = self.pending.take() {
            match self.painter.present(frame) {
                Ok(()) => {}
                Err(e) => match e {},
            }
        }
        self.frames = 0;
    }
}

impl Backend for Burst {
    type Error = ProtocolError;

    fn present(&mut self, frame: Frame) -> Result<(), Self::Error> {
        if let Some(pending) = &mut self.pending {
            if self.incremental {
                pending.coalesce_incremental(frame, 0, 2)?;
            } else {
                pending.coalesce(frame, 0)?;
            }
        } else {
            self.pending = Some(frame);
        }
        self.frames = self.frames.saturating_add(1);
        if self.frames == 8 {
            self.paint();
        }
        Ok(())
    }
}

fn measure_bursts(
    name: &str,
    updates: [&[u8]; 2],
    iterations: u64,
    incremental: bool,
) -> Result<(), Box<dyn Error>> {
    let mut ui = Ui::new(Burst {
        pending: None,
        frames: 0,
        incremental,
        painter: Painter::default(),
    });
    apply(
        &mut ui,
        &encode((("grid_resize", (1, 240, 80), (2, 240, 80)), ("flush", ()))),
    )?;
    ui.backend_mut().paint();
    let burst = |ui: &mut Ui<Burst>| -> Result<(), Box<dyn Error>> {
        for _ in 0..4 {
            for bytes in updates {
                apply(ui, bytes)?;
            }
        }
        Ok(())
    };
    for _ in 0..128 {
        burst(&mut ui)?;
    }
    ui.backend_mut().painter = Painter::default();
    // Round up to complete eight-frame bursts; the divisor is nonzero.
    let paints = iterations.div_ceil(8);
    let start = Instant::now();
    for _ in 0..paints {
        burst(&mut ui)?;
    }
    let elapsed = start.elapsed();
    let painter = &ui.backend().painter;
    let policy = if incremental { "incremental" } else { "full" };
    println!(
        "{name}, {policy}: {paints} paints (8 frames/paint), {elapsed:?} total, {:?} CPU paint; {} cells ({:.0}/paint), {} cursor updates",
        painter.elapsed,
        painter.cells,
        painter.cells as f64 / paints as f64,
        painter.cursors,
    );
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
        "{name}: {iterations} frames in {elapsed:?}; {:.0} ns/frame; {:.0} frames/s; {:.2} allocations/frame",
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
    // Immediate workloads add 1000 warmup frames, up to 1000 allocation
    // samples, and one initialization frame. Bursts add at most 1032 frames
    // for warmup, initialization, and rounding. This bound covers both.
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
    )?;

    let cursor_a = encode((("grid_cursor_goto", (1, 10, 20)), ("flush", ())));
    let cursor_b = encode((("grid_cursor_goto", (1, 10, 21)), ("flush", ())));
    let cells_a = encode((
        (
            "grid_line",
            (1, 10, 20, (("x", 0),), false),
            (2, 30, 50, (("u", 0),), false),
        ),
        ("grid_cursor_goto", (1, 10, 21)),
        ("flush", ()),
    ));
    let cells_b = encode((
        (
            "grid_line",
            (1, 10, 22, (("y", 0),), false),
            (2, 30, 52, (("v", 0),), false),
        ),
        ("grid_cursor_goto", (1, 10, 23)),
        ("flush", ()),
    ));
    for incremental in [false, true] {
        measure_bursts(
            "cursor, two 240x80 grids",
            [&cursor_a, &cursor_b],
            iterations,
            incremental,
        )?;
        measure_bursts(
            "sparse cells, two 240x80 grids",
            [&cells_a, &cells_b],
            iterations,
            incremental,
        )?;
    }
    Ok(())
}
