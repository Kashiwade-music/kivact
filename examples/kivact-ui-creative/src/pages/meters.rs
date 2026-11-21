//! Role: The page of the Creative catalog.

use super::audio_samples::*;
use super::common::*;
use catalog_shell::{Example, Page};
use kivact::prelude::{IntoView, effect};
use kivact::state::machine;
use kivact::style::style;

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
