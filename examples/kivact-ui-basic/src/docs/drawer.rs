//! Role: Catalog page of Drawer: a modal panel that is swiped away with its handle.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact::ui::ControlSize;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "drawer",
        name: "Drawer",
        description: "端に付くモーダルのパネルで、取っ手をつかんで端へスワイプして閉じます。パネルの高さ（横向きなら幅）の 30 % を越えるか、素早くはじくと閉じ、それ以外は元の位置へ戻ります。",
        usage: "use kivact::geometry::Edge;\nuse kivact::prelude::*;\nuse kivact::ui::ControlSize;\nuse kivact_ui_basic as basic;",
        source: include_str!("drawer.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "既定は下端です。取っ手を下へ引くか、Escape や背景の押下で閉じます。",
                default,
            ),
            Example::new(
                "side",
                "Side",
                "side（kivact::geometry::Edge）で付ける端と、スワイプで閉じる向きを選びます。",
                side,
            ),
        ],
    }
}

// example: default
fn default() -> impl IntoView {
    // xtask-allow(state-fields): independent: the drawer's open binding, set by Open Drawer and cleared by Submit or Cancel; the goal keeps its value while the drawer is closed, so either open state pairs with any goal
    let open = signal(false);
    // xtask-allow(state-fields): independent: the calorie goal, stepped by the minus and plus buttons in the drawer; nothing resets it on close, so every goal pairs with either drawer state
    let goal = signal(350);
    column((
        button("Open Drawer")
            .variant(ButtonVariant::Outline)
            .on_press(move || open.set(true)),
        kivact::ui::dialog("Move Goal", open)
            .mode(kivact::interaction::DialogMode::Drawer(
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
                            .style(style().font_size(36).weight(800)),
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
                .mode(kivact::interaction::DialogMode::Drawer(side))
                .variant(basic::DrawerVariant::Drawer)
                .closable(false)
                .description("Swipe the handle toward the edge to dismiss it.")
                .content(|| basic::muted("Anything can go here.")),
        ))
    })))
    .style(style().gap(8).align_items(Align::Start))
}
// end example
