//! Role: KPI probe: a large canvas over a panel, changed a little each frame.
//!
//! A 300-row panel lies under three stacked canvases: canvas A draws 4 000
//! commands, canvas B draws 1 200 rects, canvas C draws one stroked rect.
//! `MODE` picks the change of each frame: `small` (one rect of B changes
//! colour), `pan` (A moves 1 px, its outer transform), `meter` (every rect
//! of B changes height). `WIN_W` and `WIN_H` set the logical window size
//! (default 1440 x 900). Read the frame times from `KIVACT_FRAME_LOG`.

use kivact::draw::{Layer, TextFormat};
use kivact::motion::FrameRate;
use kivact::motion::frame_loop;
use kivact::prelude::*;
use kivact::style::Transform;
use kivact_bench::env_or;

/// Rects of canvas A.
const A_RECTS: u32 = 1700;
/// Stroked lines (2 px wide) of canvas A.
const A_LINES: u32 = 1500;
/// Text runs of canvas A.
const A_TEXTS: u32 = 300;
/// Rounded rects of canvas A.
const A_ROUNDED: u32 = 160;
/// Columns of canvas B's rect grid.
const B_COLS: u32 = 40;
/// Rows of canvas B's rect grid (`B_COLS` x `B_ROWS` rects).
const B_ROWS: u32 = 30;
/// Rows of the panel under the canvases.
const PANEL_ROWS: u32 = 300;
/// The colours of the rects of A and of the changing rect of B.
const PALETTE: [u32; 8] = [
    0x2563eb, 0xdc2626, 0x16a34a, 0xca8a04, 0x9333ea, 0x0891b2, 0xdb2777, 0x475569,
];
/// The colour of the rects of B that do not change colour.
const BLUE: u32 = 0x2563eb;

/// The change each frame, chosen by `MODE`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Change {
    /// One rect of canvas B changes colour.
    Small,
    /// Canvas A moves 1 px.
    Pan,
    /// Every rect of canvas B changes height.
    Meter,
}

impl Change {
    /// The change named by `MODE` (`small` when unset or unknown).
    fn from_env() -> Self {
        match env_or("MODE", String::from("small")).as_str() {
            "pan" => Change::Pan,
            "meter" => Change::Meter,
            _ => Change::Small,
        }
    }
}

/// A value in `0..1` for item `i` of the series `salt` (fixed for every frame).
fn unit(i: u32, salt: u32) -> f64 {
    let v = i
        .wrapping_mul(2_654_435_761)
        .wrapping_add(salt.wrapping_mul(97_531));
    f64::from(v % 10_000) / 10_000.0
}

/// Paint the static content of canvas A, `size` logical px large.
fn paint_a(d: &mut Draw, size: Size) {
    let (w, h) = (size.width, size.height);
    for i in 0..A_RECTS {
        let at = Point::new(unit(i, 1) * w, unit(i, 2) * h);
        let rect = Rect::new(at.x, at.y, 4.0 + unit(i, 3) * 40.0, 4.0 + unit(i, 4) * 20.0);
        d.fill_rect(rect, rgb(PALETTE[(i % 8) as usize]));
    }
    for i in 0..A_LINES {
        let from = Point::new(unit(i, 5) * w, unit(i, 6) * h);
        let to = Point::new(
            from.x + (unit(i, 7) - 0.5) * 120.0,
            from.y + (unit(i, 8) - 0.5) * 120.0,
        );
        d.line(from, to, Stroke::new(2.0), rgb(PALETTE[(i % 8) as usize]));
    }
    for i in 0..A_TEXTS {
        let at = Point::new(unit(i, 9) * w, unit(i, 10) * h);
        let format = TextFormat {
            size: 12.0,
            color: rgb(0x1f2937),
            ..Default::default()
        };
        d.text(&format!("Row {i}"), at, format);
    }
    for i in 0..A_ROUNDED {
        let at = Point::new(unit(i, 11) * w, unit(i, 12) * h);
        d.rounded_rect(Rect::new(at.x, at.y, 80.0, 24.0), 8.0, rgb(0xf1f5f9));
    }
}

/// Paint canvas B's rect grid, `size` logical px large, at frame `tick`.
fn paint_b(d: &mut Draw, size: Size, change: Change, tick: u64) {
    let cell_w = size.width / f64::from(B_COLS);
    let cell_h = size.height / f64::from(B_ROWS);
    for i in 0..B_COLS * B_ROWS {
        let col = f64::from(i % B_COLS);
        let row = f64::from(i / B_COLS);
        let full = cell_h - 4.0;
        let (height, color) = match change {
            Change::Small if i == 0 => (full, PALETTE[(tick % 8) as usize]),
            Change::Meter => {
                let step = f64::from((tick as u32).wrapping_add(i) % 10);
                (full * (0.25 + 0.75 * step / 9.0), BLUE)
            }
            Change::Small | Change::Pan => (full, BLUE),
        };
        let bottom = row * cell_h + cell_h - 2.0;
        let rect = Rect::new(col * cell_w + 2.0, bottom - height, cell_w - 4.0, height);
        d.fill_rect(rect, rgb(color));
    }
}

fn main() -> kivact::Result<()> {
    let width: f64 = env_or("WIN_W", 1440.0);
    let height: f64 = env_or("WIN_H", 900.0);
    let change = Change::from_env();
    App::new("kv_canvas")
        .window_size(width, height)
        .run(move || {
            let tick = signal(0u64);
            let looped = frame_loop(move |_| tick.update(|t| *t += 1)).essential();
            looped.start(FrameRate::Display);
            let canvas_a = canvas(move |d, size| {
                // Only the pan mode reads the tick: the other modes keep A still.
                let dx = if change == Change::Pan {
                    (tick.get() % 200) as f64
                } else {
                    0.0
                };
                d.layer(
                    Layer::new().transform(Transform::new().translate(dx, 0)),
                    |d| paint_a(d, size),
                );
            })
            .style(style().width(pct(100)).height(pct(100)));
            let canvas_b = canvas(move |d, size| {
                let tick = if change == Change::Pan { 0 } else { tick.get() };
                paint_b(d, size, change, tick);
            })
            .style(style().width(pct(100)).height(pct(100)));
            let panel = scroll(column(for_index(
                (0..PANEL_ROWS).collect::<Vec<u32>>(),
                |i, _| text(format!("Row {i}")),
            )))
            .style(style().width(pct(100)).height(pct(100)));
            // Canvas C: the same frame of one stroked rect, on top of A and B.
            let canvas_c = canvas(|d, _size| {
                d.stroke_rect(
                    Rect::new(200.0, 160.0, 400.0, 240.0),
                    Stroke::new(2.0),
                    rgb(0x1f2937),
                );
            })
            .style(style().width(pct(100)).height(pct(100)));
            stack((panel, canvas_a, canvas_b, canvas_c))
                .style(style().width(pct(100)).height(pct(100)))
        })
}
