//! Measures redraw processing and CPU cell painting, including eight-frame
//! bursts. Run with `cargo run --release -p nvim-rpc-ui --example replay --
//! 100000`. Timings are workload measurements, not a performance guarantee.
use nvim_rpc::mp::encode::{Primitive, WriterSyncExt as _};
use nvim_rpc::rpc::Message;
use nvim_rpc_ui::{Backend, Frame, Grid, GridDamage, ProtocolError, Ui};

use std::convert::Infallible;
use std::error::Error;
use std::hint::black_box;
use std::time::{Duration, Instant};

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

fn measure(name: &str, bytes: &[u8], iterations: u64) -> Result<(), Box<dyn Error>> {
    let mut ui = Ui::new(Recorder { frame: None });
    apply(
        &mut ui,
        &encode((("grid_resize", (1, 120, 40)), ("flush", ()))),
    )?;
    for _ in 0..1000 {
        apply(&mut ui, bytes)?;
    }
    let start = Instant::now();
    for _ in 0..iterations {
        apply(&mut ui, black_box(bytes))?;
    }
    let elapsed = start.elapsed();
    let frame = ui
        .into_backend()
        .frame
        .ok_or("fixture did not publish a frame")?;
    let expected = iterations.checked_add(1001).ok_or("frame count overflow")?;
    if frame.sequence != expected {
        return Err("fixture published an unexpected frame count".into());
    }
    println!(
        "{name}: {iterations} frames in {elapsed:?}; {:.0} ns/frame; {:.0} frames/s",
        elapsed.as_nanos() as f64 / iterations as f64,
        iterations as f64 / elapsed.as_secs_f64()
    );
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
    let iterations = std::env::args()
        .nth(1)
        .map(|v| v.parse::<u64>())
        .transpose()?
        .unwrap_or(100_000);
    // Bursts round up by at most seven frames, after 1024 warmup frames and
    // one initialization frame. This bound keeps every sequence within u64.
    if iterations == 0 || iterations.checked_add(1032).is_none() {
        return Err("iterations must be between 1 and u64::MAX - 1032".into());
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
    measure("typing 120x40", &typing, iterations)?;
    measure("scroll 120x40", &scroll, iterations)?;

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
