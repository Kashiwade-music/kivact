//! Catalog page of Tabs: default, disabled tab, vertical and the line variant.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::view::Orientation;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("tabs", "Tabs", "Components", include_str!("tabs.rs"))
        .description("Switches between several panels in the same area. It is the standard kivact::ui::tabs in the Lyra look (the default variant), and the left and right arrow keys move between the tabs. The line variant is .variant(basic::TabsVariant::Line); instead of a filled background it draws a line under the selected tab.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "default",
            "Default",
            "Pass the ListItem list, the selected key, and a function to show returns the panel for each selected key.",
            default,
        ))
        .example(Example::new(
            "disabled",
            "Disabled tab",
            "Set disabled on a ListItem to make a tab that cannot be chosen.",
            disabled,
        ))
        .example(Example::new(
            "vertical",
            "Vertical",
            "vertical lines the tabs up in a column; the up and down arrow keys move between them.",
            vertical,
        ))
        .example(Example::new(
            "line",
            "Line",
            ".variant(basic::TabsVariant::Line) makes the tabs the line variant. A 2 px line is drawn under the selected tab, and no background is filled.",
            line,
        ))
        .example(Example::new(
            "line-vertical",
            "Line, vertical",
            "With vertical the line sits on the right of the tabs, the up and down arrow keys move, and the panels sit to the right of the tabs.",
            line_vertical,
        ))
}

// example: default
fn default() -> impl IntoView {
    let current = signal("account");
    let options = vec![
        ListItem::new("account", "Account"),
        ListItem::new("password", "Password"),
    ];
    tabs("Account settings", options, current).content(|key| {
        let (heading, body) = match key {
            "account" => ("Account", "Make changes to your account here."),
            _ => ("Password", "Change your password here."),
        };
        column((kivact::ui::heading(heading, 4), basic::muted(body)))
            .style(style().gap(8).padding_top(16))
    })
}
// end example

// example: disabled
fn disabled() -> impl IntoView {
    let current = signal("home");
    let options = vec![
        ListItem::new("home", "Home"),
        ListItem::new("settings", "Settings").disabled(true),
        ListItem::new("about", "About"),
    ];
    tabs("Site", options, current).content(|key| {
        let (heading, body) = match key {
            "home" => ("Home", "The Settings tab is disabled."),
            _ => ("About", "Kivact is a Rust UI toolkit."),
        };
        column((kivact::ui::heading(heading, 4), basic::muted(body)))
            .style(style().gap(8).padding_top(16))
    })
}
// end example

// example: vertical
fn vertical() -> impl IntoView {
    let current = signal("overview");
    let options = vec![
        ListItem::new("overview", "Overview"),
        ListItem::new("analytics", "Analytics"),
        ListItem::new("reports", "Reports"),
    ];
    tabs("Project", options, current)
        .orientation(Orientation::Vertical)
        .content(|key| {
            let (heading, body) = match key {
                "overview" => ("Overview", "A summary of the project."),
                "analytics" => ("Analytics", "Traffic and conversions."),
                _ => ("Reports", "Monthly reports."),
            };
            column((kivact::ui::heading(heading, 4), basic::muted(body)))
                .style(style().gap(8).padding_top(16))
        })
}
// end example

// example: line
fn line() -> impl IntoView {
    let current = signal("overview");
    let options = vec![
        ListItem::new("overview", "Overview"),
        ListItem::new("analytics", "Analytics"),
        ListItem::new("reports", "Reports"),
    ];
    tabs("Project", options, current)
        .variant(basic::TabsVariant::Line)
        .content(|key| {
            let (heading, body) = match key {
                "overview" => ("Overview", "A summary of the project."),
                "analytics" => ("Analytics", "Traffic and conversions."),
                _ => ("Reports", "Monthly reports."),
            };
            column((kivact::ui::heading(heading, 4), basic::muted(body)))
                .style(style().gap(8).padding_top(16))
        })
}
// end example

// example: line-vertical
fn line_vertical() -> impl IntoView {
    let current = signal("general");
    let options = vec![
        ListItem::new("general", "General"),
        ListItem::new("security", "Security"),
        ListItem::new("billing", "Billing"),
    ];
    tabs("Settings", options, current)
        .variant(basic::TabsVariant::Line)
        .orientation(Orientation::Vertical)
        .content(|key| {
            let body = match key {
                "general" => "Name and language.",
                "security" => "Password and sessions.",
                _ => "Plans and invoices.",
            };
            column((kivact::ui::heading(key, 4), basic::muted(body)))
                .style(style().gap(8).padding_left(16))
        })
}
// end example
