//! Role: Catalog page of Table: basic, footer and selectable rows.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "table",
        name: "Table",
        description: "行と列でデータを整理して表示する表です。セルは幅を等分し、width で固定幅にできます。caption は最後に置きます。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("table.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "table_header、table_body と、右寄せの金額列です。",
                default,
            ),
            Example::new(
                "footer",
                "Footer",
                "table_footer で合計を出し、table_caption で説明を添えます。",
                footer,
            ),
            Example::new(
                "selected",
                "Selected row",
                "table_row の selected に、選択中かどうかを渡します。行を押して切り替えます。",
                selected,
            ),
        ],
    }
}

// example: default
fn default() -> impl IntoView {
    basic::table((
        basic::table_header(basic::table_row((
            basic::table_head("Invoice").width(110.0),
            basic::table_head("Status"),
            basic::table_head("Amount").align(basic::TableAlign::End),
        ))),
        basic::table_body((
            basic::table_row((
                basic::table_cell("INV001").width(110.0),
                basic::table_cell("Paid"),
                basic::table_cell("$250.00").align(basic::TableAlign::End),
            )),
            basic::table_row((
                basic::table_cell("INV002").width(110.0),
                basic::table_cell("Pending"),
                basic::table_cell("$150.00").align(basic::TableAlign::End),
            )),
        )),
    ))
    .style(style().width(460))
}
// end example

// example: footer
fn footer() -> impl IntoView {
    basic::table((
        basic::table_header(basic::table_row((
            basic::table_head("Item"),
            basic::table_head("Price").align(basic::TableAlign::End),
        ))),
        basic::table_body((
            basic::table_row((
                basic::table_cell("Keyboard"),
                basic::table_cell("$80.00").align(basic::TableAlign::End),
            )),
            basic::table_row((
                basic::table_cell("Mouse"),
                basic::table_cell("$40.00").align(basic::TableAlign::End),
            )),
        )),
        basic::table_footer(basic::table_row((
            basic::table_cell("Total"),
            basic::table_cell("$120.00").align(basic::TableAlign::End),
        ))),
        basic::table_caption("A list of recent purchases."),
    ))
    .style(style().width(460))
}
// end example

// example: selected
fn selected() -> impl IntoView {
    let chosen = signal(1usize);
    let row_for = move |index: usize, name: &'static str| {
        basic::table_row((
            basic::table_cell(name),
            basic::table_cell(format!("{} items", (index + 1) * 3)).align(basic::TableAlign::End),
        ))
        .selected(move || chosen.get() == index)
        .on_press(move || chosen.set(index))
    };
    basic::table((
        basic::table_header(basic::table_row((
            basic::table_head("Team"),
            basic::table_head("Open").align(basic::TableAlign::End),
        ))),
        basic::table_body((
            row_for(0, "Design"),
            row_for(1, "Platform"),
            row_for(2, "Growth"),
        )),
    ))
    .style(style().width(460))
}
// end example
