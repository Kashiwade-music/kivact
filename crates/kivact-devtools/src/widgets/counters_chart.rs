//! Role: The counters history: one line per counter over the samples, on a canvas.

use kivact::prelude::*;

/// The counter names in the order of a row.
const SERIES: [&str; 10] = [
    "Reactive",
    "Owners",
    "Objects",
    "Constants",
    "Nodes",
    "Timers",
    "Frame loops",
    "Animations",
    "Turns",
    "Blocking",
];
/// One color per counter (Tableau 10), the same in both schemes.
const COLORS: [u32; 10] = [
    0x4e79a7, 0xf28e2b, 0xe15759, 0x76b7b2, 0x59a14f, 0xedc948, 0xb07aa1, 0xff9da7, 0x9c755f,
    0xbab0ac,
];
/// Room around the lines.
const INSET: f64 = 8.0;

/// The lines of `rows` (one row per sample, one value per counter) and their legend.
pub(crate) fn counters_chart(rows: Value<Vec<[f64; 10]>>) -> impl IntoView {
    let legend = SERIES
        .iter()
        .zip(COLORS)
        .map(|(name, color)| {
            row((
                node().style(style().size(8, 8).background(rgb(color))),
                text(*name).style(style().font_size(11)),
            ))
            .style(style().gap(4).align_items(Align::Center))
        })
        .collect::<Vec<_>>();
    column((
        canvas(move |draw, size| rows.with(|rows| paint(draw, size, rows)))
            .name("Counters chart")
            .style(style().width(pct(100)).height(180)),
        row(legend).style(style().gap(12).wrap(true)),
    ))
    .style(style().gap(6))
}

/// One polyline per counter, all scaled to the largest value of every counter.
fn paint(draw: &mut Draw, size: Size, rows: &[[f64; 10]]) {
    let area = Rect::new(
        INSET,
        INSET,
        size.width - 2.0 * INSET,
        size.height - 2.0 * INSET,
    );
    if rows.len() < 2 || area.width <= 0.0 || area.height <= 0.0 {
        return;
    }
    let max = rows
        .iter()
        .flatten()
        .copied()
        .fold(0.0_f64, f64::max)
        .max(1.0);
    let step = area.width / (rows.len() - 1) as f64;
    for (series, color) in COLORS.iter().enumerate() {
        let mut path = Path::new();
        for (index, values) in rows.iter().enumerate() {
            let at = Point::new(
                area.x + step * index as f64,
                area.bottom() - values[series] / max * area.height,
            );
            path = if index == 0 {
                path.move_to(at)
            } else {
                path.line_to(at)
            };
        }
        draw.stroke_path(&path, &Stroke::new(1.5), rgb(*color));
    }
}
