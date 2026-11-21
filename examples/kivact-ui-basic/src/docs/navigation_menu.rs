//! Catalog page of Navigation Menu: panels with links, and plain links.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("navigation-menu", "Navigation Menu", "Components", include_str!("navigation_menu.rs"))
        .description("A row of links and expanding panels for the top of a site. A panel opens when the pointer rests on its heading, and closes when the pointer leaves. Clicking, Enter, Space or the down arrow opens it and keeps it open until Escape or a press outside. Only one panel is open at a time.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "basic",
            "Basic",
            "Pass a function to navigation_menu(...).link that makes the content of the panel each time it opens. Put navigation_menu_link items in the panel. Pressing a link closes the panel.",
            basic,
        ))
        .example(Example::new(
            "links",
            "Links",
            "navigation_menu(...).action is an item with no panel: pressing it only calls a function. It can sit beside the items with panels.",
            links,
        ))
        .example(Example::new(
            "icons",
            "Links with icons",
            "With an icon and a description, navigation_menu_link becomes a row with the icon and the description.",
            icons,
        ))
}

// example: basic
fn basic() -> impl IntoView {
    let started = || {
        column((
            kivact::ui::navigation_menu_link("Introduction")
                .description("Re-usable components built using Radix UI and Tailwind CSS."),
            kivact::ui::navigation_menu_link("Installation")
                .description("How to install dependencies and structure your app."),
            kivact::ui::navigation_menu_link("Typography")
                .description("Styles for headings, paragraphs, lists...etc"),
        ))
        .style(style().width(320))
    };
    kivact::ui::navigation_menu("Navigation")
        .link("getting-started", "Getting started", started)
        .action("docs", "Docs", || {})
        .close_delay(std::time::Duration::ZERO)
}
// end example

// example: links
fn links() -> impl IntoView {
    let clicks = signal(0usize);
    column((
        kivact::ui::navigation_menu("Navigation")
            .action("home", "Home", move || clicks.update(|n| *n += 1))
            .action("docs", "Docs", move || clicks.update(|n| *n += 1))
            .action("blog", "Blog", move || clicks.update(|n| *n += 1))
            .close_delay(std::time::Duration::ZERO),
        basic::muted(move || format!("Links pressed: {}", clicks.get())),
    ))
    .style(style().gap(12))
}
// end example

// example: icons
fn icons() -> impl IntoView {
    let components = || {
        column((
            kivact::ui::navigation_menu_link("Alert Dialog")
                .icon(icon("triangle-alert"))
                .description("A modal dialog that interrupts the user."),
            kivact::ui::navigation_menu_link("Hover Card")
                .icon(icon("eye"))
                .description("Preview content behind a link."),
            kivact::ui::navigation_menu_link("Progress").icon(icon("activity")),
        ))
        .style(style().width(300))
    };
    kivact::ui::navigation_menu("Navigation")
        .link("components", "Components", components)
        .action("list", "List", || {})
        .close_delay(std::time::Duration::ZERO)
}
// end example
