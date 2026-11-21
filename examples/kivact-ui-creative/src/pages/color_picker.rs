//! Role: The page of the Creative catalog.

use super::common::{bind, specs};
use super::editors::*;
use catalog_shell::{Example, Page};
use kivact::prelude::{IntoView, rgb, signal};
use kivact::style::{Align, style};

pub(crate) fn page() -> Page {
    Page::new(
        "color-picker",
        "Color Picker",
        "Compositing",
        include_str!("color_picker.rs"),
    )
    .description("Inline, swatch, RGB, HSV and HEX.")
    .example(Example::new("color-picker", "Color Picker", "", demo))
}

// example: color-picker
fn demo() -> impl IntoView {
    let color = signal(rgb(0x8b7cff));
    let mode = signal(Some("rgb".to_owned()));
    let hex = bind(
        move || {
            let c = color.get().components(&kivact::style::use_theme().get());
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
                let a = color
                    .get_untracked()
                    .components(&kivact::style::use_theme().get())[3];
                color.set(rgb(v).with_alpha(a));
            }
        },
    );

    specs((picker_inline(color, mode, hex), picker_popover()))
        .style(style().align_items(Align::Start))
}
// end example
