//! Role: Catalog page of Native Select: basic, sizes, disabled and invalid.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "native-select",
        name: "Native Select",
        description: "標準の見た目に近い、シンプルな選択欄です。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("native_select.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "basic",
                "Basic",
                "選択肢と Option を持つ binding を渡します。矢印キーは開かずに値を選び、押すと一覧を開きます。",
                basic,
            ),
            Example::new("sizes", "Sizes", "size の Sm は高さ 28 px です。", sizes),
            Example::new(
                "disabled",
                "Disabled",
                "disabled の選択欄は薄く表示され、開けません。",
                disabled,
            ),
            Example::new(
                "invalid",
                "Invalid",
                "invalid を true にすると、枠とリングが destructive の色になります。",
                invalid,
            ),
        ],
    }
}

// example: basic
fn fruits() -> Vec<ListItem<&'static str>> {
    vec![
        ListItem::new("apple", "Apple"),
        ListItem::new("banana", "Banana"),
        ListItem::new("blueberry", "Blueberry").disabled(true),
        ListItem::new("grapes", "Grapes"),
    ]
}

fn basic() -> impl IntoView {
    let fruit = signal(None::<&'static str>);
    basic::native_select(fruits(), fruit)
        .placeholder("Select fruit")
        .name("Fruit")
}
// end example

// example: sizes
fn sizes() -> impl IntoView {
    let (a, b) = (signal(Some("apple")), signal(Some("banana")));
    row((
        basic::native_select(vec![ListItem::new("apple", "Apple")], a).name("Default"),
        basic::native_select(vec![ListItem::new("banana", "Banana")], b)
            .size(basic::NativeSelectSize::Sm)
            .name("Small"),
    ))
    .style(style().gap(12))
}
// end example

// example: disabled
fn disabled() -> impl IntoView {
    basic::native_select(vec![ListItem::new("a", "Apple")], signal(Some("a")))
        .disabled(true)
        .name("Fruit")
}
// end example

// example: invalid
fn invalid() -> impl IntoView {
    basic::native_select(vec![ListItem::new("a", "Apple")], signal(None))
        .placeholder("Select fruit")
        .invalid(true)
        .name("Fruit")
}
// end example
