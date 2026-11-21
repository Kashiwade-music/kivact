//! Catalog page of Item: variants, media, group and pressable.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("item", "Item", "Components", include_str!("item.rs"))
        .description("A list item with an icon, a title, a description and actions. Pass on_press and the whole item can be pressed.")
        .usage("use kivact::prelude::*;\nuse kivact::ui::ControlSize;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "variants",
            "Variants",
            "The ItemVariant values Default, Outline and Muted. Put the actions in actions.",
            variants,
        ))
        .example(Example::new(
            "media",
            "Media",
            "Arrange the icon passed to media with ItemMedia's Icon variant (media_variant).",
            media,
        ))
        .example(Example::new(
            "group",
            "Group",
            "Stack the items with item_list and divide them with item_separator.",
            group,
        ))
        .example(Example::new(
            "pressable",
            "Pressable",
            "With on_press the item is a button whose background changes on hover.",
            pressable,
        ))
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
        basic::item_separator(),
        kivact::ui::item_row("Billing").media(icon("credit-card")),
        basic::item_separator(),
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
