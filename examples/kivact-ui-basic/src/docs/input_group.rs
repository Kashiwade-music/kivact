//! Role: Catalog page of Input Group: icons, text, buttons and block addons.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "input-group",
        name: "Input Group",
        description: "入力欄に前後の文字、アイコン、ボタンを付ける部品です。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("input_group.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "icon",
                "Icon and text",
                "start と end にアイコンや文字を置きます。枠とフォーカスのリングは入力欄ではなくグループが描きます。",
                icon,
            ),
            Example::new(
                "button",
                "Button",
                "input_group_button で、グループの中に押せるボタンを置きます。",
                button_addon,
            ),
            Example::new(
                "textarea",
                "Textarea",
                "block_end でテキストエリアの下に操作を並べます。",
                textarea,
            ),
            Example::new(
                "invalid",
                "Invalid",
                "invalid を true にすると、グループの枠とリングが destructive の色になります。",
                invalid,
            ),
        ],
    }
}

// example: icon
fn icon() -> impl IntoView {
    let query = signal(String::new());
    kivact::ui::input_group(
        "Search",
        (kivact::ui::text_input("Search", query)
            .variant(basic::InputGroupControl)
            .placeholder("Search..."),),
    )
    .start(kivact::view::icon("search"))
    .end(kivact::ui::input_addon("12 results"))
    .style(style().width(320))
}
// end example

// example: button
fn button_addon() -> impl IntoView {
    let url = signal(String::from("https://example.com"));
    let copied = signal(false);
    kivact::ui::input_group(
        "Link",
        (kivact::ui::text_input("Link", url).variant(basic::InputGroupControl),),
    )
    .end(
        kivact::ui::button("Copy")
            .variant(kivact::ui::ButtonVariant::Ghost)
            .variant(basic::InputGroupButtonSize::Xs)
            .style(style().margin_right(-4.8))
            .icon(kivact::view::icon(if copied.get() {
                "check"
            } else {
                "copy"
            }))
            .on_press(move || copied.set(true)),
    )
    .style(style().width(320))
}
// end example

// example: textarea
fn textarea() -> impl IntoView {
    let body = signal(String::new());
    kivact::ui::input_group(
        "Message",
        (kivact::ui::text_area("Message", body)
            .variant(basic::InputGroupControl)
            .placeholder("Write a message"),),
    )
    .block_end(kivact::ui::input_addon("0 / 280"))
    .style(style().width(320))
}
// end example

// example: invalid
fn invalid() -> impl IntoView {
    let value = signal(String::from("not an email"));
    kivact::ui::input_group(
        "Email",
        (kivact::ui::text_input("Email", value).variant(basic::InputGroupControl),),
    )
    .start(kivact::view::icon("mail"))
    .invalid(true)
    .style(style().width(320))
}
// end example
