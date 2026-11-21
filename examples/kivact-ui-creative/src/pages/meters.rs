//! Role: The page of the Creative catalog.

use catalog_shell::{Example, Page};
use super::audio_values::TransportState;
use super::audio_values::{level, smpte};
use super::common::{bind, col, line, note, spec, specs};
use super::controls::chip;
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
use super::audio_samples::*;
use super::common::*;

pub(crate) fn page() -> Page {
    Page::new("meters", "Meters", "Audio", include_str!("meters.rs"))
        .description("Peak, horizontal and loudness meters.")
        .example(Example::new("meters", "Meters", "", demo))
}

// example: meters
fn demo() -> impl IntoView {
    let state = transport();
    let integrated = machine(MeasurementState::Unmeasured, || Measurement, |_, ()| {});
    effect(move || {
        if state.get().playing {
            integrated.send(MeasurementEvent::Sample);
        }
    });
    specs((
        meters_peak_meter(state).style(style().min_height(298)),
        meters_horizontal_meter(state),
        meters_loudness(integrated),
    ))
}
// end example
