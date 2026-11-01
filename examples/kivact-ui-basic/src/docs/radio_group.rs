//! Role: Catalog page of Radio Group: basic, descriptions, horizontal and in a field.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;

pub(crate) fn page() -> Page {
    Page {
        slug: "radio-group",
        name: "Radio Group",
        description: "複数の選択肢から一つを選ぶ部品です。標準の radio_group を Lyra の見た目で描きます。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("radio_group.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "basic",
                "Basic",
                "選択肢と binding を渡します。矢印キーでフォーカスを動かすと同時に選択され、無効な項目は飛ばします。",
                basic,
            ),
            Example::new(
                "description",
                "Description",
                "ListItem の description で各項目に説明を添えます。",
                description,
            ),
            Example::new(
                "horizontal",
                "Horizontal",
                "orientation を Horizontal にすると横に並べます。",
                horizontal,
            ),
            Example::new(
                "field",
                "In a field",
                "field で包み、ラベルと説明を付けます。",
                in_field,
            ),
        ],
    }
}

// example: basic
fn basic() -> impl IntoView {
    let options = vec![
        ListItem::new("default", "Default"),
        ListItem::new("comfortable", "Comfortable"),
        ListItem::new("compact", "Compact"),
    ];
    radio_group("Density", options, signal(Some("comfortable")))
}
// end example

// example: description
fn description() -> impl IntoView {
    let options = vec![
        ListItem::new("default", "Default").description("Standard spacing."),
        ListItem::new("comfortable", "Comfortable"),
        ListItem::new("compact", "Compact").description("Fits more on screen."),
    ];
    radio_group("Density", options, signal(Some("default")))
}
// end example

// example: horizontal
fn horizontal() -> impl IntoView {
    let options = vec![
        ListItem::new("default", "Default"),
        ListItem::new("comfortable", "Comfortable"),
        ListItem::new("compact", "Compact"),
    ];
    radio_group("Density", options, signal(None)).orientation(Orientation::Horizontal)
}
// end example

// example: field
fn in_field() -> impl IntoView {
    let options = vec![
        ListItem::new("default", "Default"),
        ListItem::new("comfortable", "Comfortable"),
        ListItem::new("compact", "Compact"),
    ];
    kivact::ui::field(
        "Density",
        radio_group("Density", options, signal(Some("compact"))),
    )
    .hint("Choose how tightly rows are packed.")
}
// end example
