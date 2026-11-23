//! Catalog page of Avatar: picture, fallback, sizes, badge and group.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("avatar", "Avatar", "Components", include_str!("avatar.rs"))
        .description("An image that represents a user. Without an image it shows the fallback text (the initials). With an image it is a Role::Image, and the fallback text becomes its name.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "default",
            "Default",
            "Pass a PNG or another image (Asset) as image and it covers the fallback text. An avatar without an image keeps the initials.",
            default,
        ))
        .example(Example::new(
            "sizes",
            "Sizes",
            "The AvatarSize values Sm, Default and Lg are 24, 32 and 40 px.",
            sizes,
        ))
        .example(Example::new(
            "badge",
            "Badge",
            "badge puts a status mark at the bottom right.",
            with_badge,
        ))
        .example(Example::new(
            "group",
            "Group",
            "avatar_group overlaps the avatars by 8 px. It wraps each child, so pass a tuple of views (the kinds may differ) or a Vec. The remaining people are shown from max and total.",
            group,
        ))
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
                            .border_radius(999)
                            .background(basic::PRIMARY)
                            .color(basic::PRIMARY_FOREGROUND)
                            .outline(2, basic::BACKGROUND)
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
