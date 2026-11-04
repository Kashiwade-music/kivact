//! Role: Catalog page of Sheet: the standard dialog attached to a side of the window.
use kivact::ui::ButtonVariant;

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "sheet",
        name: "Sheet",
        description: "画面の端から出てくるモーダルのパネルです。標準の kivact::ui::dialog を DialogMode::Drawer(Side) にすると、このテーマでは sheet.tsx の見た目で描きます。題名、説明、本文、フッターを持ち、Escape、閉じるボタン、背景の押下で閉じます。",
        usage: "use kivact::geometry::Edge;\nuse kivact::prelude::*;\nuse kivact::ui::ButtonVariant;\nuse kivact::interaction::DialogMode;\nuse kivact_ui_basic as basic;",
        source: include_str!("sheet.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "DialogMode::Drawer(kivact::geometry::Edge::Right) で右端に付けます。content と footer には、開くたびに呼ばれる関数を渡します。",
                default,
            ),
            Example::new(
                "side",
                "Side",
                "DialogMode::Drawer に kivact::geometry::Edge を渡して、Top、Right、Bottom、Left のどの端に付けるかを選びます。",
                side,
            ),
            Example::new(
                "no-close-button",
                "No close button",
                "closable を false にすると、右上の閉じるボタンを出しません。フッターのボタンで閉じます。",
                no_close_button,
            ),
        ],
    }
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
            .mode(DialogMode::Drawer(kivact::geometry::Edge::Right))
            .description("Make changes to your profile here. Click save when you're done.")
            .content(move || {
                column((
                    column((
                        basic::label("Name"),
                        text_input("Name", name).style(style().width(pct(100))),
                    ))
                    .style(style().gap(8)),
                    column((
                        basic::label("Username"),
                        text_input("Username", username).style(style().width(pct(100))),
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
                .mode(DialogMode::Drawer(side))
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
            .mode(DialogMode::Drawer(kivact::geometry::Edge::Right))
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
