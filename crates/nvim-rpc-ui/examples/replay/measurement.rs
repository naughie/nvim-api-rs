use super::{allocation, fixtures::encode};

use nvim_rpc::rpc::Message;
use nvim_rpc_ui::{Backend, Frame, Ui};

use std::convert::Infallible;
use std::error::Error;
use std::hint::black_box;
use std::time::Instant;

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

pub(super) fn apply<B: Backend>(ui: &mut Ui<B>, mut bytes: &[u8]) -> Result<(), Box<dyn Error>>
where
    B::Error: Error + 'static,
{
    let Message::Notification(notification) = nvim_rpc::rpc::decode(&mut bytes)? else {
        return Err("expected a redraw notification".into());
    };
    ui.redraw(notification)?;
    Ok(())
}

pub(super) fn measure(
    name: &str,
    frames: &[Vec<u8>],
    iterations: u64,
) -> Result<(), Box<dyn Error>> {
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
    let (ret, count) = allocation::count(|| -> Result<(), Box<dyn Error>> {
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
