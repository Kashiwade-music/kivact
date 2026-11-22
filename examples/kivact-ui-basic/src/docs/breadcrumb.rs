//! Catalog page of Breadcrumb: basic trail, ellipsis and right-to-left.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("breadcrumb", "Breadcrumb", "Components", include_str!("breadcrumb.rs"))
        .description("A trail that shows the path to the current page. It is published as a Navigation named 'Breadcrumb', and its direction reverses under a right-to-left direction.")
        .usage("use kivact::prelude::*;\nuse kivact::interaction::ListItem;")
        .example(Example::new(
            "default",
            "Default",
            "Pass the keys and the names of the ListItems; on_activate receives the key of the link that was pressed.",
            default,
        ))
        .example(Example::new(
            "ellipsis",
            "Ellipsis",
            "Put basic::breadcrumb_ellipsis in the item slot to show a hidden level.",
            ellipsis,
        ))
        .example(Example::new(
            "rtl",
            "Right to left",
            "provide_direction with Rtl reverses the order and the separators.",
            rtl,
        ))
}

// example: default
fn default() -> impl IntoView {
    let at = signal(String::from("Components"));
    kivact::ui::breadcrumb(move || {
        vec![
            ListItem::new("home", "Home"),
            ListItem::new("components", "Components"),
            ListItem::new("current", at.get()),
        ]
    })
    .on_activate(move |key| at.set(if key == "home" { "Home" } else { "Components" }.into()))
    .separator(|| icon("chevron-right").style(style().size(14, 14)))
}
// end example

// example: ellipsis
fn ellipsis() -> impl IntoView {
    kivact::ui::breadcrumb(vec![
        ListItem::new("home", "Home"),
        ListItem::new("more", "More").disabled(true),
        ListItem::new("components", "Components"),
        ListItem::new("current", "Breadcrumb"),
    ])
    .on_activate(|_| {})
    .item(|item| {
        if *item.key() == "more" {
            basic::breadcrumb_ellipsis().into_view()
        } else {
            text(item.label()).into_view()
        }
    })
    .separator(|| icon("chevron-right").style(style().size(14, 14)))
}
// end example

// example: rtl
fn rtl() -> impl IntoView {
    kivact::view::provide_direction(kivact::view::ReadingDirection::Rtl, || {
        kivact::ui::breadcrumb(vec![
            ListItem::new("home", "Home"),
            ListItem::new("docs", "Docs"),
            ListItem::new("page", "Page"),
        ])
        .on_activate(|_| {})
        .separator(|| icon("chevron-left").style(style().size(14, 14)))
        .style(style().width(280))
    })
}
// end example
