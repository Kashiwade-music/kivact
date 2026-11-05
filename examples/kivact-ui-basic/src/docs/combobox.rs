//! Role: Catalog page of Combobox: basic, clear, multiple and disabled.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;

pub(crate) fn page() -> Page {
    Page {
        slug: "combobox",
        name: "Combobox",
        description: "入力で絞り込みながら選択肢から選ぶ部品です。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("combobox.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "basic",
                "Basic",
                "入力で一覧を絞り込み、矢印キーと Enter で選びます。フォーカスは入力欄に残ります。",
                basic,
            ),
            Example::new(
                "clear",
                "Clear button",
                "show_clear で選択を消すボタンを出します。empty_text は一致なしのときの文言です。",
                clear,
            ),
            Example::new(
                "multiple",
                "Multiple",
                "Vec の binding を渡すと複数を選べ、選んだ項目を chip として並べて × で外せます。",
                multiple,
            ),
            Example::new(
                "disabled",
                "Disabled",
                "disabled の間は入力できません。",
                disabled,
            ),
        ],
    }
}

// example: basic
fn basic() -> impl IntoView {
    let items = vec![
        ListItem::new("next", "Next.js"),
        ListItem::new("nuxt", "Nuxt.js"),
        ListItem::new("remix", "Remix"),
        ListItem::new("astro", "Astro"),
    ];
    kivact::ui::combobox("Framework", items, signal(None))
        .placeholder("Select a framework")
        .name("Framework")
        .style(style().width(260))
}
// end example

// example: clear
fn clear() -> impl IntoView {
    let items = vec![
        ListItem::new("next", "Next.js"),
        ListItem::new("nuxt", "Nuxt.js"),
        ListItem::new("remix", "Remix"),
        ListItem::new("astro", "Astro"),
    ];
    kivact::ui::combobox("Framework", items, signal(Some("remix")))
        .show_clear(true)
        .empty_text("No frameworks found.")
        .name("Framework")
        .style(style().width(260))
}
// end example

// example: multiple
fn multiple() -> impl IntoView {
    let items = vec![
        ListItem::new("next", "Next.js"),
        ListItem::new("nuxt", "Nuxt.js"),
        ListItem::new("remix", "Remix"),
        ListItem::new("astro", "Astro"),
    ];
    kivact::ui::combobox("Frameworks", items, signal(vec!["next", "astro"])).chips()
        .placeholder("Add framework")
        .name("Frameworks")
        .style(style().width(360))
}
// end example

// example: disabled
fn disabled() -> impl IntoView {
    let items = vec![
        ListItem::new("next", "Next.js"),
        ListItem::new("nuxt", "Nuxt.js"),
        ListItem::new("remix", "Remix"),
        ListItem::new("astro", "Astro"),
    ];
    kivact::ui::combobox("Framework", items, signal(Some("nuxt")))
        .disabled(true)
        .name("Framework")
        .style(style().width(260))
}
// end example
