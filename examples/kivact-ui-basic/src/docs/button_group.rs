//! Role: Catalog page of Button Group: horizontal, vertical, separator and text.
use kivact::ui::ButtonVariant;

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;

pub(crate) fn page() -> Page {
    Page {
        slug: "button-group",
        name: "Button Group",
        description: "関連するボタンを隣り合わせにまとめる部品です。2 つ目からの子は前の子と接する枠線を消すので、境界の線は 1 本になります。子を 1 つずつ整えるため、button_group には 1 つの view ではなく、子の tuple（種類が違ってもよい）か Vec を渡します。",
        usage: "use kivact::prelude::*;\nuse kivact::ui::ButtonVariant;\nuse kivact_ui_basic as basic;",
        source: include_str!("button_group.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "button_group にボタンの tuple を渡します。",
                default,
            ),
            Example::new(
                "vertical",
                "Vertical",
                "orientation に Orientation::Vertical を渡すと縦に積みます。区切りの線も向きに合わせて横線になります。",
                vertical,
            ),
            Example::new(
                "separator",
                "Separator",
                "button_group_separator でボタンの間に --input の色の線を入れます。枠のない Secondary のボタンを区切るときに使います。横に並べたグループでは縦線になります。",
                separator,
            ),
            Example::new(
                "text",
                "Text",
                "button_group_text は、グループの中の操作ではない文字の区画です（shadcn の ButtonGroupText）。URL の欄の前の https:// や、数値の後ろの単位のような前置き・後置きに使います。グループが高さをそろえるので、隣のボタンや入力欄と同じ高さで、文字は縦の中央に来ます。",
                with_text,
            ),
        ],
    }
}

// example: default
fn default() -> impl IntoView {
    kivact::ui::button_group("Actions", (
        button("Archive").variant(ButtonVariant::Outline),
        button("Report").variant(ButtonVariant::Outline),
        button("Snooze").variant(ButtonVariant::Outline),
    ))
}
// end example

// example: vertical
fn vertical() -> impl IntoView {
    kivact::ui::button_group("Actions", (
        button("Top").variant(ButtonVariant::Outline),
        kivact::ui::button_group_separator(),
        button("Middle").variant(ButtonVariant::Outline),
        button("Bottom").variant(ButtonVariant::Outline),
    ))
    .orientation(Orientation::Vertical)
}
// end example

// example: separator
fn separator() -> impl IntoView {
    kivact::ui::button_group("Actions", (
        button("Copy").variant(ButtonVariant::Secondary),
        kivact::ui::button_group_separator(),
        button("Paste").variant(ButtonVariant::Secondary),
    ))
}
// end example

// example: text
fn with_text() -> impl IntoView {
    let url = signal(String::from("example.com"));
    kivact::ui::button_group("Actions", (
        kivact::ui::button_group_text("https://"),
        text_input("URL", url).style(style().width(160)),
        button("Go").variant(ButtonVariant::Outline),
    ))
}
// end example
