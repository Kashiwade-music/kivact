//! Role: Catalog page of Toggle Group: multiple, single, outline and vertical.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact::ui::ControlSize;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "toggle-group",
        name: "Toggle Group",
        description: "関連するトグルをまとめ、一つまたは複数を選ぶ部品です。",
        usage: "use kivact::prelude::*;\nuse kivact::ui::ControlSize;\nuse kivact_ui_basic as basic;",
        source: include_str!("toggle_group.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "multiple",
                "Multiple",
                "Vec を受ける toggle_group は複数を同時に押せます。矢印キーでフォーカスを動かし、Space で押します。",
                multiple,
            ),
            Example::new(
                "single",
                "Single",
                "Option を受ける toggle_group は高々一つを押せます。もう一度押すと外れます。",
                single,
            ),
            Example::new(
                "sizes",
                "Spacing and size",
                "spacing は 4 px 単位の間隔、size は全項目の大きさです。",
                spacing,
            ),
            Example::new(
                "vertical",
                "Vertical",
                "orientation を Vertical にすると縦に並べます。",
                vertical,
            ),
        ],
    }
}

// example: multiple
fn multiple() -> impl IntoView {
    let items = vec![
        ListItem::new("bold", "Bold").with_icon("bold"),
        ListItem::new("italic", "Italic").with_icon("italic"),
        ListItem::new("underline", "Underline").with_icon("underline"),
    ];
    kivact::ui::toggle_group("Options", items, signal(vec!["bold"]))
        .icon_only(true)
        .name("Text style")
}
// end example

// example: single
fn single() -> impl IntoView {
    let items = vec![
        ListItem::new("left", "Left"),
        ListItem::new("center", "Center"),
        ListItem::new("right", "Right"),
    ];
    kivact::ui::toggle_group("Options", items, signal(Some("left")))
        .variant(basic::ToggleVariant::Outline)
        .spacing(0.0)
        .name("Alignment")
}
// end example

// example: sizes
fn spacing() -> impl IntoView {
    let items = vec![
        ListItem::new("bold", "Bold"),
        ListItem::new("italic", "Italic"),
        ListItem::new("underline", "Underline"),
    ];
    kivact::ui::toggle_group("Options", items, signal(vec!["italic"]))
        .size(ControlSize::Small)
        .spacing(1.0)
        .name("Text style")
}
// end example

// example: vertical
fn vertical() -> impl IntoView {
    let items = vec![
        ListItem::new("bold", "Bold"),
        ListItem::new("italic", "Italic"),
        ListItem::new("underline", "Underline"),
    ];
    kivact::ui::toggle_group("Options", items, signal(None))
        .variant(basic::ToggleVariant::Outline)
        .orientation(Orientation::Vertical)
        .spacing(0.0)
        .name("Text style")
}
// end example
