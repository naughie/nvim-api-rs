//! Measures redraw processing, allocations, and damage-directed CPU painting.
//! Run with `cargo run --release -p nvim-rpc-ui --example replay -- 100000`.

mod allocation;
mod fixtures;
mod measurement;
mod painting;

use nvim_rpc_ui::Coalescing;

use std::error::Error;

#[global_allocator]
static ALLOCATOR: allocation::Allocator = allocation::Allocator;

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
    for workload in fixtures::immediate() {
        let workload = workload?;
        measurement::measure(workload.name, &workload.frames, iterations)?;
    }
    for (name, [first, second]) in fixtures::bursts() {
        for coalescing in [Coalescing::FullRepaint, Coalescing::Incremental] {
            painting::measure(name, [&first, &second], iterations, coalescing)?;
        }
    }
    Ok(())
}
