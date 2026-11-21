//! Role: The page of the Creative catalog.

use super::audio_samples::*;
use super::common::*;
use catalog_shell::{Example, Page};
use kivact::prelude::{IntoView, signal};
use kivact::style::style;

pub(crate) fn page() -> Page {
    Page::new(
        "transport",
        "Transport",
        "Audio",
        include_str!("transport.rs"),
    )
    .description("Play, record, locators and LCD.")
    .example(Example::new("transport", "Transport", "", demo))
}

// example: transport
fn demo() -> impl IntoView {
    let state = transport();
    let format = signal(0_usize);
    specs((
        transport_section_transport_panel(state).style(style().min_height(245)),
        transport_section_lcd_time_display(state, format),
        transport_section_locators(state),
    ))
}
// end example
