//! Role: Frame-time probe: `ROWS` static items plus one text written every display frame.
//!
//! `MODE` picks the item: `row` (text + button, default), `text`, `box`
//! (a styled empty box), `button`, `checkbox`. `STILL=1` stops the frame
//! loop (idle memory). Read the frame times from `KIVACT_FRAME_LOG`.

use kivact::motion::FrameRate;
use kivact::prelude::*;
use kivact_bench::env_or;

fn item(mode: &str, i: usize) -> View {
    match mode {
        "text" => text(format!("Row {i}")).into_view(),
        "box" => node().style(style().height(20).width(200)).into_view(),
        "button" => button("x").into_view(),
        "checkbox" => checkbox(format!("Row {i}"), signal(false)).into_view(),
        _ => row((text(format!("Row {i}")), button("x")))
            .style(style().gap(4))
            .into_view(),
    }
}

fn main() -> kivact::Result<()> {
    let rows: u32 = env_or("ROWS", 1000);
    let mode: String = env_or("MODE", "row".to_owned());
    let still = std::env::var_os("STILL").is_some();
    App::new("kv_stress").run(move || {
        let tick = signal(0u64);
        let looped = frame_loop(move |_| tick.update(|t| *t += 1)).essential();
        if !still {
            looped.start(FrameRate::Display);
        }
        let mode = mode.clone();
        column((
            text(move || format!("Tick {}", tick.get())),
            scroll(column(for_index(
                (0..rows).collect::<Vec<u32>>(),
                move |i, _| item(&mode, i),
            ))),
        ))
    })
}
