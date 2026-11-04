//! Role: Catalog page of Avatar: picture, fallback, sizes, badge and group.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "avatar",
        name: "Avatar",
        description: "ユーザーを表す画像。画像がないときは fallback の文字（頭文字）を表示します。画像を渡すと Role::Image になり、fallback の文字がその名前になります。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("avatar.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "image に PNG などの画像（Asset）を渡すと、fallback の文字の上に画像が重なります。画像を渡さないアバターは、頭文字のままです。",
                default,
            ),
            Example::new(
                "sizes",
                "Sizes",
                "AvatarSize の Sm、Default、Lg が 24、32、40 px です。",
                sizes,
            ),
            Example::new(
                "badge",
                "Badge",
                "badge で右下に状態を示す印を置けます。",
                with_badge,
            ),
            Example::new(
                "group",
                "Group",
                "avatar_group は 8 px ずつ重ねます。子を 1 つずつ包むため、view の tuple（種類が違ってもよい）か Vec で渡します。avatar_group_count で残りの人数を示します。",
                group,
            ),
        ],
    }
}

// example: default
fn default() -> impl IntoView {
    let photo: &'static [u8] = include_bytes!("../../assets/images/avatar.png");
    row((
        basic::avatar("CN").image(photo),
        basic::avatar("ER"),
        basic::avatar("LR"),
    ))
    .style(style().gap(12).align_items(Align::Center))
}
// end example

// example: sizes
fn sizes() -> impl IntoView {
    let photo: &'static [u8] = include_bytes!("../../assets/images/avatar.png");
    row((
        basic::avatar("SM").size(basic::AvatarSize::Sm).image(photo),
        basic::avatar("MD").image(photo),
        basic::avatar("LG").size(basic::AvatarSize::Lg).image(photo),
    ))
    .style(style().gap(12).align_items(Align::Center))
}
// end example

// example: badge
fn with_badge() -> impl IntoView {
    row((
        basic::avatar("CN").badge(node()),
        basic::avatar("LR")
            .size(basic::AvatarSize::Lg)
            .badge(icon("plus")),
    ))
    .style(style().gap(12).align_items(Align::Center))
}
// end example

// example: group
fn group() -> impl IntoView {
    let photo: &'static [u8] = include_bytes!("../../assets/images/avatar.png");
    basic::avatar_group((
        basic::avatar("CN").image(photo),
        basic::avatar("LR"),
        basic::avatar("ER"),
        basic::avatar_group_count("+3"),
    ))
}
// end example
