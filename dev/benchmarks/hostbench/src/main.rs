//! Role: Host benchmark with public API.
//!
//! Host benchmark: scenarios written with the public `kivact` API only,
//! on the Arvane host (`ARVANE_RENDERER=cpu` makes it draw on the CPU
//! instead of the GPU, reported as the host `arvane-cpu`).
//!
//! `hostbench <scenario>` opens one window, drives itself (frame loops,
//! timers, `NodeRef::scroll_into_view`; no OS input), prints `MARK <name>
//! <unix_ns>` lines at its phase boundaries and one `RESULT {json}` line,
//! and exits. Scenarios: `startup`, `scroll_virtual`, `scroll_column`,
//! `animation`, `text`, `churn_1k`, `churn_10k`, `churn_50k`,
//! `relayout_10k`, `relayout_50k`, `resize`, `idle`, and the writes of a
//! pointer drag or hover (`drag_backdrop`, `drag_slider`, `hover_tooltip`:
//! `pointer.rs`; the public API sends no pointer input, so they make the
//! writes a move makes, once per frame).
//!
//! It measures the engine and Arvane in the same scenes, on the hosts
//! `arvane` (the default GPU renderer) and `arvane-cpu`.
//!
//! Presentation timing comes from the host's frame log: run with
//! `KIVACT_FRAME_LOG=<file>` (one line per presented frame, see
//! `kivact-engine`'s `FrameLog`). Without it the frame fields are empty.
//! A line holds the wall clock time the frame was presented at and the
//! host's UI thread time for it: the redraw's layout, scene, raster and
//! present (on the GPU, raster is encoding and submitting, present is
//! waiting for the surface's buffer and presenting). From the log the
//! bench reports, per phase, the frame count, the intervals, the host's
//! times and the delay from a change to the next frame (`report.rs`).
//!
//! The runner (`dev/tools/gui-harness/hostbench.ps1`) builds the release
//! binary, runs each scenario three times per host (hosts alternating),
//! samples the process's CPU time and memory (working set, private bytes
//! and their peaks) at the marks, times the process's creation to its
//! first frame, drives `resize` (`SetWindowPos`: the public API cannot
//! resize a window) and writes a table of medians to
//! `target/gui-evidence/hostbench/<time>/summary.md` and the raw results to
//! `raw.jsonl` there. It holds the GUI lock, brings the window to the
//! front and attaches no UI Automation client:
//!
//! ```text
//! powershell -NoProfile -ExecutionPolicy Bypass -File dev\tools\gui-harness\hostbench.ps1 [-Runs 3] [-Hosts arvane,arvane-cpu] [-Scenarios text,idle] [-NoBuild]
//! ```
//!
//! Caveats: while a UI Automation client runs anywhere on the system, the
//! host keeps its accessibility tree up (updated after the frame that
//! follows a layout), and the summary says so for every run. Windows
//! updates process CPU time in 15.6 ms steps. A run with accessibility off
//! does not measure the cost under assistive technology.
//!
//! Comparing performance:
//! - Keep the base (`main`) commit, the candidate's commit (or a record of
//!   its uncommitted source), the Rust toolchain, the release profile and
//!   the executables' SHA-256. Build apart from measuring, on the same
//!   machine, renderer, viewport, foreground state and accessibility
//!   client state.
//! - Keep at least three raw values per GPU and CPU host, their median and
//!   spread; read relayout's turn, layout, scene, raster and present,
//!   process CPU, private and working set and frame delays apart.
//! - Keep the first and any later batch apart; never overwrite them with a
//!   more favorable run. Overlapping ranges prove no equality, and a
//!   measured difference alone proves no cause or general speedup; moving
//!   code between crates alone is not a cost's cause. Compare a
//!   reorganization's difference and a regression from the latest `main`
//!   separately, and later changes under the same conditions.
//! - Raw data, executables' hashes and run conditions stay under the
//!   ignored `target/` and are not committed.
//!
//! The driver state (tick counters, row refs) lives in `Rc`/`Cell`: it is
//! measurement plumbing, not app state.

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "the benchmark reports its measurements on stdout"
)]

mod churn;
mod pointer;
mod report;
mod scenes;
mod scroll;

use kivact::prelude::*;
use kivact::view::View;

const SCENARIOS: &[&str] = &[
    "startup",
    "scroll_virtual",
    "scroll_column",
    "animation",
    "text",
    "churn_1k",
    "churn_10k",
    "churn_50k",
    "relayout_10k",
    "relayout_50k",
    "resize",
    "idle",
    "drag_backdrop",
    "drag_slider",
    "hover_tooltip",
];

fn scene(name: &str) -> View {
    match name {
        "startup" => scenes::startup(),
        "scroll_virtual" => scroll::scroll_list(true),
        "scroll_column" => scroll::scroll_list(false),
        "animation" => scenes::animation(),
        "text" => scenes::text_heavy(),
        "churn_1k" => churn::churn(1_000, false),
        "churn_10k" => churn::churn(10_000, false),
        "churn_50k" => churn::churn(50_000, false),
        "relayout_10k" => churn::churn(10_000, true),
        "relayout_50k" => churn::churn(50_000, true),
        "resize" => scenes::resize(),
        "drag_backdrop" => pointer::drag_backdrop(),
        "drag_slider" => pointer::drag_slider(),
        "hover_tooltip" => pointer::hover_tooltip(),
        _ => scenes::idle(),
    }
}

fn main() -> kivact::Result<()> {
    report::start_clock();
    let name = std::env::args().nth(1).unwrap_or_default();
    if !SCENARIOS.contains(&name.as_str()) {
        eprintln!(
            "usage: hostbench <scenario>  (one of: {})",
            SCENARIOS.join(", ")
        );
        std::process::exit(2);
    }
    App::new(format!("hostbench {name}")).run(move || scene(&name))
}
