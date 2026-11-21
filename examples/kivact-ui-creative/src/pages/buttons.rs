//! Role: The page of the Creative catalog.

use super::common::specs;
use super::controls::*;
use catalog_shell::{Example, Page};
use kivact::prelude::{IntoView, signal};
use kivact::style::GridPlacement;
use kivact::style::style;
use kivact_ui_creative as ui;

pub(crate) fn page() -> Page {
    Page::new("buttons", "Buttons", "Controls", include_str!("buttons.rs"))
        .description("Push, icon, toggle, segmented and tool palette buttons.")
        .example(Example::new("buttons", "Buttons", "", demo))
}

// example: buttons
fn demo() -> impl IntoView {
    let status = signal(String::new());
    let snap = signal(true);
    let interpolation = signal(Some("bezier".to_owned()));
    let curve = signal(ui::CubicBezier::default());
    specs((
        buttons_static_collections(),
        buttons_standard_indicators(),
        buttons_cubic_easing(curve),
        buttons_push_button(status),
        buttons_glyph_button(),
        buttons_toggle_button(snap),
        buttons_segmented_enum_row(interpolation),
        buttons_split_menu_button(status),
        buttons_tool_palette().style(style().grid_column(GridPlacement::span(2)).min_height(360)),
    ))
}
// end example
