//! Role: The page of the Creative catalog.

use super::audio_samples::*;
use super::audio_values::TransportState;
use super::audio_values::{level, smpte};
use super::common::*;
use super::common::{bind, col, line, note, spec, specs};
use super::controls::chip;
use catalog_shell::{Example, Page};
use kivact::geometry::Point;
use kivact::geometry::Rect;
use kivact::interaction::ScalarMapping;
use kivact::prelude::{
    Element, IntoView, Signal, column, effect, grid, node, rgb, row, signal, spacer, text,
};
use kivact::state::IntoText;
use kivact::state::{Machine, Step, machine};
use kivact::style::{Align, Color, Justify, pct, style};
use kivact::style::{GridPlacement, Track};
use kivact::view::{Node, Orientation};
use kivact_ui_creative as ui;

pub(crate) fn page() -> Page {
    Page::new(
        "knobs-faders",
        "Knobs & Faders",
        "Audio",
        include_str!("knobs_faders.rs"),
    )
    .description("Knobs, faders and ranges.")
    .example(Example::new("knobs-faders", "Knobs & Faders", "", demo))
}

// example: knobs-faders
fn demo() -> impl IntoView {
    let range = signal((24.0, 180.0));
    specs((
        knobs_rotary_knob().style(style().grid_column(GridPlacement::span(2)).min_height(240)),
        knobs_channel_fader(),
        knobs_horizontal_range(range),
    ))
}
// end example
