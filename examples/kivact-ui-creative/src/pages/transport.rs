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
    Page::new("transport", "Transport", "Audio", include_str!("transport.rs"))
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
