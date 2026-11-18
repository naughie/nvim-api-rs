//! Measures redraw processing and backend presentation while retaining the
//! previous frame. Run with `cargo run --release -p nvim-rpc-ui --example replay
//! -- 100000`. Timings are workload measurements, not a performance guarantee.
use nvim_rpc::mp::encode::{Primitive, WriterSyncExt as _};
use nvim_rpc::rpc::Message;
use nvim_rpc_ui::{Backend, Frame, Ui};

use std::convert::Infallible;
use std::error::Error;
use std::hint::black_box;
use std::time::Instant;

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
    if iterations == 0 || iterations.checked_add(1001).is_none() {
        return Err("iterations must be between 1 and u64::MAX - 1001".into());
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
    measure("scroll 120x40", &scroll, iterations)
}
