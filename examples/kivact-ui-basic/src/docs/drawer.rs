//! Catalog page of Drawer: a modal panel that is swiped away with its handle.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("drawer", "Drawer", "Components", include_str!("drawer.rs"))
        .description("A modal panel that comes from an edge; you grab the handle and swipe it to the edge to close it. It closes when you pass 30 % of the panel's height (its width when it is sideways) or flick it, and otherwise it returns to its place.")
        .usage("use kivact::geometry::Edge;\nuse kivact::prelude::*;\nuse kivact::ui::ControlSize;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "default",
            "Default",
            "The default edge is the bottom. Pull the handle down, or press Escape or the background, to close it.",
            default,
        ))
        .example(Example::new(
            "side",
            "Side",
            "side (kivact::geometry::Edge) chooses the edge it comes from and the direction of the swipe that closes it.",
            side,
        ))
}

// example: default
// xtask-allow(state): independent: open is the drawer binding set by Open Drawer and cleared by Submit or Cancel; goal is changed only by the plus and minus buttons inside the drawer and is kept, not reverted, when it closes
fn default() -> impl IntoView {
    let open = signal(false);
    let goal = signal(350);
    column((
        button("Open Drawer")
            .variant(ButtonVariant::Outline)
            .on_press(move || open.set(true)),
        kivact::ui::dialog("Move Goal", open)
            .mode(kivact::ui::DialogKind::Drawer(
                kivact::geometry::Edge::Bottom,
            ))
            .variant(basic::DrawerVariant::Drawer)
            .closable(false)
            .description("Set your daily activity goal.")
            .content(move || {
                row((
                    button("Decrease")
                        .variant(ButtonVariant::Outline)
                        .hide_label(true)
                        .icon(icon("minus"))
                        .size(ControlSize::Small)
                        .on_press(move || goal.update(|goal| *goal -= 10)),
                    column((
                        text(move || goal.get().to_string())
                            .style(style().font_size(36).font_weight(800)),
                        basic::muted("Calories/day"),
                    ))
                    .style(style().align_items(Align::Center).min_width(120)),
                    button("Increase")
                        .variant(ButtonVariant::Outline)
                        .hide_label(true)
                        .icon(icon("plus"))
                        .size(ControlSize::Small)
                        .on_press(move || goal.update(|goal| *goal += 10)),
                ))
                .style(
                    style()
                        .align_items(Align::Center)
                        .justify_content(Justify::Center),
                )
            })
            .footer(move || {
                (
                    button("Submit").on_press(move || open.set(false)),
                    button("Cancel")
                        .variant(ButtonVariant::Outline)
                        .on_press(move || open.set(false)),
                )
            }),
    ))
}
// end example

// example: side
fn side() -> impl IntoView {
    let edges = [
        ("Top", kivact::geometry::Edge::Top),
        ("Right", kivact::geometry::Edge::Right),
        ("Bottom", kivact::geometry::Edge::Bottom),
        ("Left", kivact::geometry::Edge::Left),
    ];
    row(Vec::from(edges.map(|(label, side)| {
        let open = signal(false);
        column((
            button(label)
                .variant(ButtonVariant::Outline)
                .on_press(move || open.set(true)),
            kivact::ui::dialog(format!("{label} drawer"), open)
                .mode(kivact::ui::DialogKind::Drawer(side))
                .variant(basic::DrawerVariant::Drawer)
                .closable(false)
                .description("Swipe the handle toward the edge to dismiss it.")
                .content(|| basic::muted("Anything can go here.")),
        ))
    })))
    .style(style().gap(8).align_items(Align::Start))
}
// end example
