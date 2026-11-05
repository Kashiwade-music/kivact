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
                "avatar_group は 8 px ずつ重ねます。子を 1 つずつ包むため、view の tuple（種類が違ってもよい）か Vec で渡します。max と total から残りの人数を表示します。",
                group,
            ),
        ],
    }
}

// example: default
fn default() -> impl IntoView {
    let photo: &'static [u8] = include_bytes!("../../assets/images/avatar.png");
    row((
        kivact::ui::avatar("CN")
            .fallback_content(|| text("CN"))
            .src(Some(kivact::view::Asset::from(photo))),
        kivact::ui::avatar("ER").fallback_content(|| text("ER")),
        kivact::ui::avatar("LR").fallback_content(|| text("LR")),
    ))
    .style(style().gap(12).align_items(Align::Center))
}
// end example

// example: sizes
fn sizes() -> impl IntoView {
    let photo: &'static [u8] = include_bytes!("../../assets/images/avatar.png");
    row((
        kivact::ui::avatar("SM")
            .fallback_content(|| text("SM"))
            .size(kivact::ui::ControlSize::Small)
            .src(Some(kivact::view::Asset::from(photo))),
        kivact::ui::avatar("MD")
            .fallback_content(|| text("MD"))
            .src(Some(kivact::view::Asset::from(photo))),
        kivact::ui::avatar("LG")
            .fallback_content(|| text("LG"))
            .size(kivact::ui::ControlSize::Large)
            .src(Some(kivact::view::Asset::from(photo))),
    ))
    .style(style().gap(12).align_items(Align::Center))
}
// end example

// example: badge
fn with_badge() -> impl IntoView {
    row((
        kivact::ui::avatar("CN")
            .fallback_content(|| text("CN"))
            .status(Some(kivact::interaction::Tone::Accent)),
        kivact::ui::avatar("LR")
            .fallback_content(|| text("LR"))
            .size(kivact::ui::ControlSize::Large)
            .status_content(
                node()
                    .style(
                        style()
                            .position(kivact::style::Position::Absolute)
                            .right(0)
                            .bottom(0)
                            .size(12, 12)
                            .radius(999)
                            .background(basic::ColorName::Primary)
                            .color(basic::ColorName::PrimaryForeground)
                            .ring(2, basic::ColorName::Background)
                            .z_index(10),
                    )
                    .child(icon("plus").style(style().size(12, 12))),
            ),
    ))
    .style(style().gap(12).align_items(Align::Center))
}
// end example

// example: group
fn group() -> impl IntoView {
    let photo: &'static [u8] = include_bytes!("../../assets/images/avatar.png");
    kivact::ui::avatar_group((
        kivact::ui::avatar("CN")
            .fallback_content(|| text("CN"))
            .src(Some(kivact::view::Asset::from(photo))),
        kivact::ui::avatar("LR").fallback_content(|| text("LR")),
        kivact::ui::avatar("ER").fallback_content(|| text("ER")),
    ))
    .max(3usize)
    .total(6usize)
}
// end example
