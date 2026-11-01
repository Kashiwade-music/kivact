//! Role: Catalog page of Slider: single, range, step and vertical.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact::ui::ScalarRange;
use kivact::ui::range_slider;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "slider",
        name: "Slider",
        description: "範囲の中から値を選ぶスライダーです。標準の slider と range_slider を Lyra の見た目で描きます。",
        usage: "use kivact::prelude::*;\nuse kivact::ui::range_slider;",
        source: include_str!("slider.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "basic",
                "Basic",
                "名前と f64 の binding を渡します。トラックのどこを押しても、つまみをドラッグしても値が変わります。",
                basic,
            ),
            Example::new(
                "range",
                "Range",
                "range_slider は二つのつまみで最小と最大を選びます。つまみは交差しません。",
                range,
            ),
            Example::new(
                "step",
                "Step",
                "range に ScalarRange を渡して、範囲と刻みを決めます。キーも同じ刻みで動きます。",
                step,
            ),
            Example::new(
                "vertical",
                "Vertical",
                "vertical(true) で縦向きになり、下が最小です。",
                vertical,
            ),
        ],
    }
}

// example: basic
fn basic() -> impl IntoView {
    let volume = signal(50.0);
    column((
        slider("Volume", volume).style(style().width(320)),
        basic::muted(move || format!("Volume: {}", volume.get())),
    ))
    .style(style().gap(12))
}
// end example

// example: range
fn range() -> impl IntoView {
    let (low, high) = (signal(25.0), signal(75.0));
    column((
        range_slider("Price", low, high).style(style().width(320)),
        basic::muted(move || format!("{} - {}", low.get(), high.get())),
    ))
    .style(style().gap(12))
}
// end example

// example: step
fn step() -> impl IntoView {
    let range = ScalarRange::new(10.0, 30.0)
        .and_then(|range| range.with_step(0.5))
        .expect("a valid range");
    slider("Temperature", signal(20.0))
        .range(range)
        .style(style().width(320))
}
// end example

// example: vertical
fn vertical() -> impl IntoView {
    row((
        slider("Left", signal(40.0)).orientation(Orientation::Vertical),
        slider("Right", signal(70.0)).orientation(Orientation::Vertical),
    ))
    .style(style().gap(24).height(160))
}
// end example
