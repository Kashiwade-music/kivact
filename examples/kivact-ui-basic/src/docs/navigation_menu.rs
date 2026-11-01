//! Role: Catalog page of Navigation Menu: panels with links, and plain links.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "navigation-menu",
        name: "Navigation Menu",
        description: "サイトの上部に置く、リンクと展開パネルの並びです。見出しにポインタを置くとパネルが開き、離れると閉じます。クリック、Enter、Space、下矢印で開くと、Escape か外側の押下まで開いたままです。パネルは同時に 1 つだけ開きます。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("navigation_menu.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "basic",
                "Basic",
                "NavItem::menu に、開くたびにパネルの中身を作る関数を渡します。パネルの中は navigation_menu_link を並べます。リンクを押すとパネルが閉じます。",
                basic,
            ),
            Example::new(
                "links",
                "Links",
                "NavItem::link は、パネルを持たずに押された時の関数を呼ぶだけの項目です。パネルの項目と並べられます。",
                links,
            ),
            Example::new(
                "icons",
                "Links with icons",
                "navigation_menu_link に icon と description を付けると、アイコンと説明の付いた行になります。",
                icons,
            ),
        ],
    }
}

// example: basic
fn basic() -> impl IntoView {
    let started = || {
        column((
            basic::navigation_menu_link("Introduction")
                .description("Re-usable components built using Radix UI and Tailwind CSS."),
            basic::navigation_menu_link("Installation")
                .description("How to install dependencies and structure your app."),
            basic::navigation_menu_link("Typography")
                .description("Styles for headings, paragraphs, lists...etc"),
        ))
        .style(style().width(320))
    };
    basic::navigation_menu(vec![
        basic::NavItem::menu("Getting started", started),
        basic::NavItem::link("Docs", || {}),
    ])
}
// end example

// example: links
fn links() -> impl IntoView {
    let clicks = signal(0usize);
    column((
        basic::navigation_menu(vec![
            basic::NavItem::link("Home", move || clicks.update(|n| *n += 1)),
            basic::NavItem::link("Docs", move || clicks.update(|n| *n += 1)),
            basic::NavItem::link("Blog", move || clicks.update(|n| *n += 1)),
        ]),
        basic::muted(move || format!("Links pressed: {}", clicks.get())),
    ))
    .style(style().gap(12))
}
// end example

// example: icons
fn icons() -> impl IntoView {
    let components = || {
        column((
            basic::navigation_menu_link("Alert Dialog")
                .icon(icon("triangle-alert"))
                .description("A modal dialog that interrupts the user."),
            basic::navigation_menu_link("Hover Card")
                .icon(icon("eye"))
                .description("Preview content behind a link."),
            basic::navigation_menu_link("Progress").icon(icon("activity")),
        ))
        .style(style().width(300))
    };
    basic::navigation_menu(vec![
        basic::NavItem::menu("Components", components),
        basic::NavItem::link("List", || {}),
    ])
}
// end example
