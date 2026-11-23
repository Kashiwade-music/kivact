//! Role: The page of the Creative catalog.

use super::audio_samples::*;
use super::common::specs;
use catalog_shell::{Example, Page};
use kivact::prelude::{IntoView, signal};
use kivact::style::GridPlacement;
use kivact::style::style;

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
