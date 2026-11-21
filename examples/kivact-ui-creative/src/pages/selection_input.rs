//! Role: The page of the Creative catalog.

use super::common::specs;
use super::controls::*;
use catalog_shell::{Example, Page};
use kivact::prelude::{IntoView, signal};
use kivact::style::style;

pub(crate) fn page() -> Page {
    Page::new(
        "selection-input",
        "Selection & Input",
        "Controls",
        include_str!("selection_input.rs"),
    )
    .description("Checkboxes, switches, enums, text and data block selectors.")
    .example(Example::new(
        "selection-input",
        "Selection & Input",
        "",
        demo,
    ))
}

// example: selection-input
fn demo() -> impl IntoView {
    let radio = signal(Some("median".to_owned()));
    specs((
        selection_checkbox_radio(radio).style(style().min_height(190)),
        selection_switch(),
        selection_enum_dropdown(),
        selection_text_field(),
        selection_data_block_selector(),
        selection_script_expression().style(style().min_height(202)),
    ))
}
// end example
