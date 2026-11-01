//! Role: Catalog page of Label: a label for an input and a small form.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "label",
        name: "Label",
        description: "入力部品に名前を付けるラベルです。target を渡すと、ラベルを押して入力欄へフォーカスを移せます。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("label.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "with-input",
                "With input",
                "NodeRef を入力欄とラベルの target で共有します。ラベルを押すと入力欄にフォーカスが移ります。",
                with_input,
            ),
            Example::new(
                "form",
                "In a form",
                "ラベルと入力欄の組を縦に並べたフォームの例です。ラベルは 12 px の文字で、行間を取りません。",
                form,
            ),
        ],
    }
}

// example: with-input
fn with_input() -> impl IntoView {
    let field = node_ref();
    let email = signal(String::new());
    column((
        basic::label("Email address").target(field),
        text_input("Email address", email)
            .placeholder("you@example.com")
            .node_ref(field)
            .style(style().width(280)),
    ))
    .style(style().gap(8))
}
// end example

// example: form
fn form() -> impl IntoView {
    let first = node_ref();
    let last = node_ref();
    column((
        column((
            basic::label("First name").target(first),
            text_input("First name", signal(String::new()))
                .node_ref(first)
                .style(style().width(280)),
        ))
        .style(style().gap(8)),
        column((
            basic::label("Last name").target(last),
            text_input("Last name", signal(String::new()))
                .node_ref(last)
                .style(style().width(280)),
        ))
        .style(style().gap(8)),
    ))
    .style(style().gap(16))
}
// end example
