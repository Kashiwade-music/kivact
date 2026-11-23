//! Catalog page of Button: variants, sizes, icons, loading and disabled.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use std::time::Duration;

pub(crate) fn page() -> Page {
    Page::new("button", "Button", "Components", include_str!("button.rs"))
        .description("A button that runs an action. It is the standard kivact::ui::button in the Lyra look, with a kind, a size, an icon and a loading state.")
        .usage("use kivact::prelude::*;\nuse kivact::ui::ControlSize;")
        .example(Example::new(
            "variants",
            "Variants",
            "Choose the look with variant. Primary is shadcn's default, and Danger is destructive.",
            variants,
        ))
        .example(Example::new(
            "sizes",
            "Sizes",
            "ControlSize ExtraSmall, Small, Medium and Large give the heights xs, sm, default and lg (24, 28, 32 and 36 px).",
            sizes,
        ))
        .example(Example::new(
            "with-icon",
            "With icon",
            "icon and trailing_icon add a Lucide icon; the space beside the icon is 2 px narrower. A button with an icon and hide_label(true) is a square button that keeps only its name.",
            with_icon,
        ))
        .example(Example::new(
            "loading",
            "Loading",
            "While loading is true a spinner is shown and the button cannot be pressed. A press makes it loading for 2 seconds.",
            loading,
        ))
        .example(Example::new(
            "disabled",
            "Disabled",
            "A disabled button is drawn faint and cannot be pressed.",
            disabled,
        ))
}

// example: variants
fn variants() -> impl IntoView {
    row((
        button("Default"),
        button("Secondary").variant(ButtonVariant::Secondary),
        button("Outline").variant(ButtonVariant::Outline),
        button("Ghost").variant(ButtonVariant::Ghost),
        button("Destructive").variant(ButtonVariant::Danger),
        button("Link").variant(ButtonVariant::Link),
    ))
    .style(style().gap(8))
}
// end example

// example: sizes
fn sizes() -> impl IntoView {
    column((
        row((
            button("Extra small")
                .variant(ButtonVariant::Outline)
                .size(ControlSize::ExtraSmall),
            button("Small")
                .variant(ButtonVariant::Outline)
                .size(ControlSize::Small),
            button("Default").variant(ButtonVariant::Outline),
            button("Large")
                .variant(ButtonVariant::Outline)
                .size(ControlSize::Large),
        ))
        .style(style().gap(8).align_items(Align::Center)),
        row((
            button("Add")
                .variant(ButtonVariant::Outline)
                .hide_label(true)
                .icon(icon("plus"))
                .size(ControlSize::ExtraSmall),
            button("Add")
                .variant(ButtonVariant::Outline)
                .hide_label(true)
                .icon(icon("plus"))
                .size(ControlSize::Small),
            button("Add")
                .variant(ButtonVariant::Outline)
                .hide_label(true)
                .icon(icon("plus")),
            button("Add")
                .variant(ButtonVariant::Outline)
                .hide_label(true)
                .icon(icon("plus"))
                .size(ControlSize::Large),
        ))
        .style(style().gap(8).align_items(Align::Center)),
    ))
    .style(style().gap(16).align_items(Align::Center))
}
// end example

// example: with-icon
fn with_icon() -> impl IntoView {
    row((
        button("Send")
            .variant(ButtonVariant::Outline)
            .icon(icon("mail")),
        button("Continue").trailing(icon("arrow-right")),
        button("Delete")
            .hide_label(true)
            .icon(icon("trash-2"))
            .variant(ButtonVariant::Danger),
        button("Settings")
            .hide_label(true)
            .icon(icon("settings"))
            .variant(ButtonVariant::Ghost),
    ))
    .style(style().gap(8))
}
// end example

// example: loading
fn loading() -> impl IntoView {
    let busy = signal(false);
    row((
        button("Please wait")
            .variant(ButtonVariant::Outline)
            .busy(true),
        button("Save changes").busy(busy).on_press(move || {
            busy.set(true);
            spawn::<()>(async move {
                sleep(Duration::from_secs(2)).await;
                busy.set(false);
            });
        }),
    ))
    .style(style().gap(8))
}
// end example

// example: disabled
fn disabled() -> impl IntoView {
    row((
        button("Default").disabled(true),
        button("Outline")
            .variant(ButtonVariant::Outline)
            .disabled(true),
        button("Secondary")
            .variant(ButtonVariant::Secondary)
            .disabled(true),
        button("Destructive")
            .variant(ButtonVariant::Danger)
            .disabled(true),
    ))
    .style(style().gap(8))
}
// end example
