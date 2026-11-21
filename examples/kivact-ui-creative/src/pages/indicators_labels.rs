//! Role: The page of the Creative catalog.

use catalog_shell::{Example, Page};
use super::common::{col, line, note, spec, specs};
use super::controls::chip;
use kivact::prelude::{Element, IntoView, node, rgb, row, show, signal, spacer, text};
use kivact::style::{Align, style};
use kivact::style::{Overflow, TextAlign};
use kivact::view::Node;
use super::indicators::*;
use super::common::*;

pub(crate) fn page() -> Page {
    Page::new("indicators-labels", "Indicators & Labels", "Controls", include_str!("indicators_labels.rs"))
        .description("LEDs, status, keycaps, labels and cache indicators.")
        .example(Example::new("indicators-labels", "Indicators & Labels", "", demo))
}

// example: indicators-labels
fn demo() -> impl IntoView {
    let picked = signal(0usize);
    let colors = [
        ("Red", 0xb53838),
        ("Yellow", 0xe4d84c),
        ("Aqua", 0xa9cbc7),
        ("Pink", 0xe5bcc9),
        ("Lavender", 0xa9a9ca),
        ("Peach", 0xe7c19e),
        ("Sea Foam", 0xb3c7b3),
        ("Blue", 0x7c8cff),
        ("Green", 0x4aa44c),
        ("Purple", 0x8e2c9a),
        ("Orange", 0xe8920d),
        ("Brown", 0x7f452a),
        ("Fuchsia", 0xf46dd6),
        ("Cyan", 0x3ad3e0),
        ("Sandstone", 0xa89677),
        ("Dark Green", 0x1e401e),
    ];
    specs((
        indicators_led().style(style().min_height(167)),
        indicators_status_chip(),
        indicators_shortcuts(),
        indicators_label_colors(picked, colors),
        indicators_cache_render_bar(),
        indicators_badges_on_controls().style(style().min_height(116)),
    ))
}
// end example
