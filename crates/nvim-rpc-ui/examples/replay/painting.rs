use super::{fixtures::encode, measurement::apply};

use nvim_rpc_ui::{Backend, Coalescing, Frame, Grid, GridDamage, ProtocolError, Ui};

use std::error::Error;
use std::hint::black_box;
use std::time::{Duration, Instant};

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

    fn paint(&mut self, frame: Frame) {
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
    }
}

struct Burst {
    pending: Option<Frame>,
    frames: u8,
    coalescing: Coalescing,
    painter: Painter,
}

impl Burst {
    fn paint(&mut self) {
        if let Some(frame) = self.pending.take() {
            self.painter.paint(frame);
        }
        self.frames = 0;
    }
}

impl Backend for Burst {
    type Error = ProtocolError;

    fn present(&mut self, frame: Frame) -> Result<(), Self::Error> {
        if let Some(pending) = &mut self.pending {
            match self.coalescing {
                Coalescing::FullRepaint => pending.coalesce(frame, 0)?,
                Coalescing::Incremental => pending.coalesce_incremental(frame, 0, 2)?,
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

pub(super) fn measure(
    name: &str,
    updates: [&[u8]; 2],
    iterations: u64,
    coalescing: Coalescing,
) -> Result<(), Box<dyn Error>> {
    let mut ui = Ui::new(Burst {
        pending: None,
        frames: 0,
        coalescing,
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
    let policy = match coalescing {
        Coalescing::FullRepaint => "full",
        Coalescing::Incremental => "incremental",
    };
    println!(
        "{name}, {policy}: {paints} paints (8 frames/paint), {elapsed:?} total, {:?} CPU paint; {} cells ({:.0}/paint), {} cursor updates",
        painter.elapsed,
        painter.cells,
        painter.cells as f64 / paints as f64,
        painter.cursors,
    );
    Ok(())
}
