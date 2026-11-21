//! Role: The page of the Creative catalog.

use catalog_shell::{Example, Page};
use super::audio_values::TransportState;
use kivact::state::Signal;
use super::common::{bind, col, keyed_options, line, note, spec, specs};
use super::controls::num;
use super::controls::tool_palette;
use super::editor_images as images;
use kivact::draw::Path;
use kivact::geometry::Rect;
use kivact::geometry::{Point, Size};
use kivact::interaction::MenuEntry;
use kivact::prelude::{
    Element, IntoView, NodeRef, Role, dynamic, memo, node, rgb, row, signal, spacer, text,
};
use kivact::style::Position;
use kivact::style::{Align, Color, Justify, pct, style};
use kivact::view::Node;
use kivact_ui_creative as ui;
use super::editors::*;
use super::common::*;

pub(crate) fn page() -> Page {
    Page::new("color-picker", "Color Picker", "Compositing", include_str!("color_picker.rs"))
        .description("Inline, swatch, RGB, HSV and HEX.")
        .example(Example::new("color-picker", "Color Picker", "", demo))
}

// example: color-picker
fn demo() -> impl IntoView {
    let color = signal(rgb(0x8b7cff));
    let mode = signal(Some("rgb".to_owned()));
    let hex = bind(
        move || {
            let c = color.get().components();
            format!(
                "#{:02X}{:02X}{:02X}",
                (c[0] * 255.0).round() as u8,
                (c[1] * 255.0).round() as u8,
                (c[2] * 255.0).round() as u8
            )
        },
        move |value: String| {
            if let Ok(v) = u32::from_str_radix(value.trim().trim_start_matches('#'), 16)
                && v <= 0xffffff
            {
                let a = color.get_untracked().components()[3];
                color.set(rgb(v).with_alpha(a));
            }
        },
    );

    specs((picker_inline(color, mode, hex), picker_popover()))
        .style(style().align_items(Align::Start))
}
// end example
