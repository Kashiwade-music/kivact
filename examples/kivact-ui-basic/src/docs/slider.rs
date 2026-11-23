//! Catalog page of Slider: single, range, step and vertical.

use catalog_shell::{Example, Page};
use kivact::interaction::ScalarRange;
use kivact::prelude::*;
use kivact::ui::range_slider;
use kivact::view::Orientation;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("slider", "Slider", "Components", include_str!("slider.rs"))
        .description("A slider that chooses a value within a range. It draws the standard slider and range_slider in the Lyra look.")
        .usage("use kivact::prelude::*;\nuse kivact::ui::range_slider;")
        .example(Example::new(
            "basic",
            "Basic",
            "Pass a name and an f64 binding. The value changes when you press anywhere on the track or drag the thumb.",
            basic,
        ))
        .example(Example::new(
            "range",
            "Range",
            "range_slider chooses a minimum and a maximum with two thumbs. The thumbs do not cross.",
            range,
        ))
        .example(Example::new(
            "step",
            "Step",
            "Pass a ScalarRange as range to set the range and the step. The keys move by the same step.",
            step,
        ))
        .example(Example::new(
            "vertical",
            "Vertical",
            "vertical(true) makes it vertical, with the minimum at the bottom.",
            vertical,
        ))
}

// example: basic
fn basic() -> impl IntoView {
    let volume = signal(50.0);
    column((
        slider(volume).label("Volume").style(style().width(320)),
        basic::muted(move || format!("Volume: {}", volume.get())),
    ))
    .style(style().gap(12))
}
// end example

// example: range
fn range() -> impl IntoView {
    let (low, high) = (signal(25.0), signal(75.0));
    column((
        range_slider(low, high)
            .label("Price")
            .style(style().width(320)),
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
    slider(signal(20.0))
        .label("Temperature")
        .range(range)
        .style(style().width(320))
}
// end example

// example: vertical
fn vertical() -> impl IntoView {
    row((
        slider(signal(40.0))
            .label("Left")
            .orientation(Orientation::Vertical),
        slider(signal(70.0))
            .label("Right")
            .orientation(Orientation::Vertical),
    ))
    .style(style().gap(24).height(160))
}
// end example
