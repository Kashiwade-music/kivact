//! Role: Startup probe: a counter window that stamps main entry and root setup.
//!
//! `KIVACT_FRAME_LOG` gives the end of the first presented frame; the
//! startup KPI is that time minus `T main`. `WIN_W`/`WIN_H` set the
//! logical window size (default 800 x 600).

use kivact::prelude::*;
use kivact_bench::{env_or, stamp};

fn counter() -> impl IntoView {
    stamp("root");
    on_mount(|| stamp("mounted"));
    let count = signal(0);
    column((
        text(move || format!("Count: {}", count.get())),
        button("Increment").on_press(move || count.update(|n| *n += 1)),
    ))
    .style(style().padding(24).gap(12))
}

fn main() -> kivact::Result<()> {
    stamp("main");
    let width: f64 = env_or("WIN_W", 800.0);
    let height: f64 = env_or("WIN_H", 600.0);
    App::new("kv_startup")
        .window_size(width, height)
        .run(counter)
}
