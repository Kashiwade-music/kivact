//! Catalog page of Table: basic, footer and selectable rows.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("table", "Table", "Components", include_str!("table.rs"))
        .description("A table that arranges data in rows and columns. Cells divide the width equally, and width fixes a width. The caption goes last.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "default",
            "Default",
            "table_header, table_body and an amount column aligned to the right.",
            default,
        ))
        .example(Example::new(
            "footer",
            "Footer",
            "table_footer shows a total, and table_caption adds a description.",
            footer,
        ))
        .example(Example::new(
            "selected",
            "Selected row",
            "Pass whether the row is selected to selected of table_row. A press on a row switches it.",
            selected,
        ))
}

// example: default
fn default() -> impl IntoView {
    kivact::ui::table((
        kivact::ui::table_header((kivact::ui::table_row((
            kivact::ui::table_head("Invoice").width(110.0),
            kivact::ui::table_head("Status"),
            kivact::ui::table_head("Amount").align(kivact::style::TextAlign::End),
        )),)),
        kivact::ui::table_body((
            kivact::ui::table_row((
                kivact::ui::table_cell("INV001").width(110.0),
                kivact::ui::table_cell("Paid"),
                kivact::ui::table_cell("$250.00").align(kivact::style::TextAlign::End),
            )),
            kivact::ui::table_row((
                kivact::ui::table_cell("INV002").width(110.0),
                kivact::ui::table_cell("Pending"),
                kivact::ui::table_cell("$150.00").align(kivact::style::TextAlign::End),
            )),
        )),
    ))
    .style(style().width(460))
}
// end example

// example: footer
fn footer() -> impl IntoView {
    kivact::ui::table((
        kivact::ui::table_header((kivact::ui::table_row((
            kivact::ui::table_head("Item"),
            kivact::ui::table_head("Price").align(kivact::style::TextAlign::End),
        )),)),
        kivact::ui::table_body((
            kivact::ui::table_row((
                kivact::ui::table_cell("Keyboard"),
                kivact::ui::table_cell("$80.00").align(kivact::style::TextAlign::End),
            )),
            kivact::ui::table_row((
                kivact::ui::table_cell("Mouse"),
                kivact::ui::table_cell("$40.00").align(kivact::style::TextAlign::End),
            )),
        )),
        kivact::ui::table_footer((kivact::ui::table_row((
            kivact::ui::table_cell("Total"),
            kivact::ui::table_cell("$120.00").align(kivact::style::TextAlign::End),
        )),)),
        basic::table_caption("A list of recent purchases."),
    ))
    .style(style().width(460))
}
// end example

// example: selected
fn selected() -> impl IntoView {
    let chosen = signal(1usize);
    let row_for = move |index: usize, name: &'static str| {
        kivact::ui::table_row((
            kivact::ui::table_cell(name),
            kivact::ui::table_cell(format!("{} items", (index + 1) * 3))
                .align(kivact::style::TextAlign::End),
        ))
        .with_node(|node| node.selected(move || chosen.get() == index))
        .on_press(move || chosen.set(index))
    };
    kivact::ui::table((
        kivact::ui::table_header((kivact::ui::table_row((
            kivact::ui::table_head("Team"),
            kivact::ui::table_head("Open").align(kivact::style::TextAlign::End),
        )),)),
        kivact::ui::table_body((
            row_for(0, "Design"),
            row_for(1, "Platform"),
            row_for(2, "Growth"),
        )),
    ))
    .style(style().width(460))
}
// end example
