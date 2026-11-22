//! Catalog page of Sheet: the standard dialog attached to a side of the window.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::ui::DialogKind;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("sheet", "Sheet", "Components", include_str!("sheet.rs"))
        .description("A modal panel that slides in from the edge of the screen. Make the standard kivact::ui::dialog a DialogKind::Drawer(Side) and this theme draws it in the look of sheet.tsx. It has a title, a description, a body and a footer, and it closes on Escape, the close button or a press on the background.")
        .usage("use kivact::geometry::Edge;\nuse kivact::prelude::*;\nuse kivact::ui::DialogKind;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "default",
            "Default",
            "DialogKind::Drawer(kivact::geometry::Edge::Right) attaches it at the right edge. Pass content and footer as functions that are called each time it opens.",
            default,
        ))
        .example(Example::new(
            "side",
            "Side",
            "Pass a kivact::geometry::Edge to DialogKind::Drawer to choose the edge it attaches to: Top, Right, Bottom or Left.",
            side,
        ))
        .example(Example::new(
            "no-close-button",
            "No close button",
            "closable(false) hides the close button at the top right. Close it with a footer button.",
            no_close_button,
        ))
}

// example: default
fn default() -> impl IntoView {
    let open = signal(false);
    let name = signal(String::from("Pedro Duarte"));
    let username = signal(String::from("@peduarte"));
    column((
        button("Open")
            .variant(ButtonVariant::Outline)
            .on_press(move || open.set(true)),
        dialog("Edit profile", open)
            .mode(DialogKind::Drawer(kivact::geometry::Edge::Right))
            .description("Make changes to your profile here. Click save when you're done.")
            .content(move || {
                column((
                    column((
                        basic::label("Name"),
                        text_input(name)
                            .label("Name")
                            .style(style().width(pct(100))),
                    ))
                    .style(style().gap(8)),
                    column((
                        basic::label("Username"),
                        text_input(username)
                            .label("Username")
                            .style(style().width(pct(100))),
                    ))
                    .style(style().gap(8)),
                ))
                .style(style().gap(16))
            })
            .footer(move || {
                (
                    button("Save changes").on_press(move || open.set(false)),
                    button("Close")
                        .variant(ButtonVariant::Outline)
                        .on_press(move || open.set(false)),
                )
            }),
    ))
}
// end example

// example: side
fn side() -> impl IntoView {
    let sides = [
        ("Top", kivact::geometry::Edge::Top),
        ("Right", kivact::geometry::Edge::Right),
        ("Bottom", kivact::geometry::Edge::Bottom),
        ("Left", kivact::geometry::Edge::Left),
    ];
    row(Vec::from(sides.map(|(label, side)| {
        let open = signal(false);
        column((
            button(label)
                .variant(ButtonVariant::Outline)
                .on_press(move || open.set(true)),
            dialog(format!("{label} sheet"), open)
                .mode(DialogKind::Drawer(side))
                .description("This sheet is attached to one edge of the window.")
                .content(|| basic::muted("Press Escape or the close button to dismiss it.")),
        ))
    })))
    .style(style().gap(8).align_items(Align::Start))
}
// end example

// example: no-close-button
fn no_close_button() -> impl IntoView {
    let open = signal(false);
    column((
        button("Open")
            .variant(ButtonVariant::Outline)
            .on_press(move || open.set(true)),
        dialog("Terms of service", open)
            .mode(DialogKind::Drawer(kivact::geometry::Edge::Right))
            .closable(false)
            .description("Please read the terms before you continue.")
            .footer(move || {
                button("I understand")
                    .variant(ButtonVariant::Secondary)
                    .on_press(move || open.set(false))
            }),
    ))
}
// end example
