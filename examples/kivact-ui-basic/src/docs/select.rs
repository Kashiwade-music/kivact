//! Role: Catalog page of Select: basic, sizes, disabled item and with icon.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact::ui::ControlSize;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "select",
        name: "Select",
        description: "選択肢の一覧から一つを選びます。標準の kivact::ui::select に Lyra の見た目を当てたもので、選択はキーとして呼び出し側が持ちます。",
        usage: "use kivact::prelude::*;\nuse kivact::ui::ControlSize;\nuse kivact_ui_basic as basic;",
        source: include_str!("select.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "basic",
                "Basic",
                "ListItem の一覧と Option のキーを渡します。矢印キーで選択肢を移動し、Enter で決めます。",
                basic,
            ),
            Example::new(
                "sizes",
                "Sizes",
                "size に ControlSize::Small を渡すと、高さ 28 px の小さい選択欄になります。",
                sizes,
            ),
            Example::new(
                "disabled-item",
                "Disabled item",
                "ListItem の disabled で、選べない選択肢を作れます。",
                disabled_item,
            ),
            Example::new(
                "with-icon",
                "With icon",
                "icon で選択欄の先頭に Lucide のアイコンを添えます。",
                with_icon,
            ),
        ],
    }
}

// example: basic
fn basic() -> impl IntoView {
    let fruit = signal(None::<&'static str>);
    let options = vec![
        ListItem::new("apple", "Apple"),
        ListItem::new("banana", "Banana"),
        ListItem::new("blueberry", "Blueberry"),
        ListItem::new("grapes", "Grapes"),
        ListItem::new("pineapple", "Pineapple"),
    ];
    column((
        select("Fruit", options, fruit)
            .placeholder("Select a fruit")
            .style(style().width(200)),
        basic::muted(move || format!("Selected: {}", fruit.get().unwrap_or("none"))),
    ))
    .style(style().gap(8))
}
// end example

// example: sizes
fn sizes() -> impl IntoView {
    let options = vec![
        ListItem::new("apple", "Apple"),
        ListItem::new("banana", "Banana"),
        ListItem::new("grapes", "Grapes"),
    ];
    column((
        select("Small fruit", options.clone(), signal(Some("apple")))
            .size(ControlSize::Small)
            .style(style().width(200)),
        select("Default fruit", options, signal(Some("banana"))).style(style().width(200)),
    ))
    .style(style().gap(12))
}
// end example

// example: disabled-item
fn disabled_item() -> impl IntoView {
    let options = vec![
        ListItem::new("apple", "Apple"),
        ListItem::new("banana", "Banana"),
        ListItem::new("blueberry", "Blueberry").disabled(true),
        ListItem::new("grapes", "Grapes").disabled(true),
        ListItem::new("pineapple", "Pineapple"),
    ];
    select("Fruit", options, signal(None::<&'static str>))
        .placeholder("Select a fruit")
        .style(style().width(200))
}
// end example

// example: with-icon
fn with_icon() -> impl IntoView {
    let places = vec![
        ListItem::new("home", "Home"),
        ListItem::new("work", "Work"),
        ListItem::new("travel", "Travel"),
    ];
    select("Place", places, signal(Some("home")))
        .icon(icon("map-pin"))
        .style(style().width(200))
}
// end example
