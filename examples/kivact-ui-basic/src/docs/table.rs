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
