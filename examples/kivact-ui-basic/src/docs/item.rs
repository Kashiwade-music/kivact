//! Role: Catalog page of Item: variants, media, group and pressable.
use kivact::ui::ButtonVariant;

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact::ui::ControlSize;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "item",
        name: "Item",
        description: "アイコン、題名、説明、操作を持つ一覧の項目です。on_press を渡すと項目全体が押せます。",
        usage: "use kivact::prelude::*;\nuse kivact::ui::ButtonVariant;\nuse kivact::ui::ControlSize;\nuse kivact_ui_basic as basic;",
        source: include_str!("item.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "variants",
                "Variants",
                "ItemVariant の Default、Outline、Muted です。actions に操作を置きます。",
                variants,
            ),
            Example::new(
                "media",
                "Media",
                "media に渡したアイコンを、media_variant の Icon で整えます。",
                media,
            ),
            Example::new(
                "group",
                "Group",
                "item_group で縦に並べ、item_separator で区切ります。",
                group,
            ),
            Example::new(
                "pressable",
                "Pressable",
                "on_press を渡すと、ホバーで背景が変わるボタンになります。",
                pressable,
            ),
        ],
    }
}

// example: variants
fn variants() -> impl IntoView {
    let action = || {
        button("Action")
            .variant(ButtonVariant::Outline)
            .size(ControlSize::Small)
    };
    kivact::ui::item_list((
        kivact::ui::item_row("Default")
            .description("A plain item.")
            .actions(action()),
        kivact::ui::item_row("Outline")
            .variant(basic::ItemVariant::Outline)
            .description("A bordered item.")
            .actions(action()),
        kivact::ui::item_row("Muted")
            .variant(basic::ItemVariant::Muted)
            .description("A muted item.")
            .actions(action()),
    ))
    .style(style().width(460))
}
// end example

// example: media
fn media() -> impl IntoView {
    kivact::ui::item_row("Security alert")
        .variant(basic::ItemVariant::Outline)
        .media(icon("shield"))
        .variant(basic::ItemMediaVariant::Icon)
        .description("New login from an unknown device.")
        .style(style().width(460))
}
// end example

// example: group
fn group() -> impl IntoView {
    kivact::ui::item_list((
        kivact::ui::item_row("Profile").media(icon("user")),
        node()
            .role(Role::Separator)
            .orientation(Orientation::Horizontal)
            .style(
                style()
                    .width(pct(100))
                    .height(1)
                    .shrink(0)
                    .margin_y(8)
                    .background(basic::ColorName::Border),
            ),
        kivact::ui::item_row("Billing").media(icon("credit-card")),
        node()
            .role(Role::Separator)
            .orientation(Orientation::Horizontal)
            .style(
                style()
                    .width(pct(100))
                    .height(1)
                    .shrink(0)
                    .margin_y(8)
                    .background(basic::ColorName::Border),
            ),
        kivact::ui::item_row("Notifications").media(icon("bell")),
    ))
    .style(style().width(460))
}
// end example

// example: pressable
fn pressable() -> impl IntoView {
    let count = signal(0);
    kivact::ui::item_row("Open settings")
        .variant(basic::ItemVariant::Outline)
        .media(icon("settings"))
        .variant(basic::ItemMediaVariant::Icon)
        .description(move || format!("Opened {} times", count.get()))
        .actions(icon("chevron-right"))
        .on_press(move || count.update(|n| *n += 1))
        .style(style().width(460))
}
// end example
