//! Catalog page of Popover: the dialog at a trigger, its placement and alignment.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::ui::DialogKind;
use kivact::view::OverlayAlign;
use kivact::view::Placement;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("popover", "Popover", "Components", include_str!("popover.rs"))
        .description("A panel that is not modal and opens over the page near its trigger. Make the standard kivact::ui::dialog a DialogKind::Popover and attach it, and this theme draws it in the look of popover.tsx. It closes on Escape, on a press outside and when focus leaves, and closing returns focus to the trigger. The button passed to attach is the open and close button.")
        .usage("use kivact::prelude::*;\nuse kivact::ui::DialogKind;\nuse kivact::view::{OverlayAlign, Placement};\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "default",
            "Default",
            "Pass the open signal and the title, make the content with description and content, and attach it to the trigger. When it opens, focus moves to the first focusable element.",
            default,
        ))
        .example(Example::new(
            "placement",
            "Placement",
            "placement (kivact::view::Placement) chooses the side it opens on; when it does not fit it turns to the other side. To hide the title, set hide_title(true); the title then serves only as the name for assistive technology.",
            placement,
        ))
        .example(Example::new(
            "align",
            "Align",
            "align (kivact::view::OverlayAlign) chooses whether it lines up with the start, the center or the end of the trigger. The default is the start.",
            align,
        ))
}

// example: default
fn default() -> impl IntoView {
    let open = signal(false);
    let width = signal(String::from("100%"));
    let height = signal(String::from("25px"));
    let field = |label: &'static str, value: Signal<String>| {
        row((
            basic::label(label).style(style().width(80)),
            text_input(value).label(label).style(style().flex_grow(1)),
        ))
        .style(style().gap(8).align_items(Align::Center))
    };
    dialog("Dimensions", open)
        .mode(DialogKind::Popover)
        .description("Set the dimensions for the layer.")
        .content(move || {
            column((field("Width", width), field("Height", height)))
                .style(style().gap(8).width(pct(100)))
        })
        .attach(button("Open popover").variant(ButtonVariant::Outline))
}
// end example

// example: placement
fn placement() -> impl IntoView {
    let sides = [
        ("Top", Placement::Above),
        ("Right", Placement::End),
        ("Bottom", Placement::Below),
        ("Left", Placement::Start),
    ];
    row(Vec::from(sides.map(|(label, placement)| {
        let open = signal(false);
        dialog(format!("{label} popover"), open)
            .mode(DialogKind::Popover)
            .hide_title(true)
            .placement(placement)
            .content(move || text(format!("Opens on the {label} side.")))
            .attach(button(label).variant(ButtonVariant::Outline))
    })))
    .style(style().gap(8).align_items(Align::Center))
}
// end example

// example: align
fn align() -> impl IntoView {
    let aligns = [
        ("Start", OverlayAlign::Start),
        ("Center", OverlayAlign::Center),
        ("End", OverlayAlign::End),
    ];
    row(Vec::from(aligns.map(|(label, align)| {
        let open = signal(false);
        dialog(label, open)
            .mode(DialogKind::Popover)
            .align(align)
            .description("Lined up with the trigger.")
            .attach(button(label).variant(ButtonVariant::Outline))
    })))
    .style(style().gap(8).align_items(Align::Center))
}
// end example
