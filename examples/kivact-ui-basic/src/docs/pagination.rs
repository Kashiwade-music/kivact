//! Catalog page of Pagination: pages helper, manual links, labels and long lists.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::ui::PageItem;

pub(crate) fn page() -> Page {
    Page::new("pagination", "Pagination", "Components", include_str!("pagination.rs"))
        .description("Moves through a long list by page number. pagination builds the whole control from a binding of the current page (counted from 0) and the total page count: the previous and next moves and the ellipses. label_format changes how page numbers are written.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "pages",
            "Pages",
            "Shows the first and last pages and the current page with its neighbors; at the ends, previous and next are disabled.",
            pages,
        ))
        .example(Example::new(
            "manual",
            "Custom cell contents",
            "Even if the item slot changes how a cell looks, moving and selecting use the same page binding.",
            manual,
        ))
        .example(Example::new(
            "labels",
            "Page labels",
            "label_format receives the page number counted from 1 and returns the text and the name of the link.",
            labels,
        ))
        .example(Example::new(
            "long",
            "Many pages",
            "Even with 100 pages, only the area around the current page is shown.",
            long,
        ))
}

// example: pages
fn pages() -> impl IntoView {
    let page = signal(0usize);
    column((
        kivact::ui::pagination(5usize, page).item(|item| match item {
            PageItem::Previous => row((icon("chevron-left"), text("Previous"))).into_view(),
            PageItem::Next => row((text("Next"), icon("chevron-right"))).into_view(),
            PageItem::Page(n) => text((n + 1).to_string()).into_view(),
            _ => icon("ellipsis").into_view(),
        }),
        text(move || format!("Page {} of 5", page.get() + 1)).style(style().font_size(12)),
    ))
    .style(style().gap(12).align_items(Align::Center))
}
// end example

// example: manual
fn manual() -> impl IntoView {
    let page = signal(1usize);
    kivact::ui::pagination(10usize, page)
        .siblings(0usize)
        .item(|item| match item {
            PageItem::Previous => row((icon("chevron-left"), text("Previous"))).into_view(),
            PageItem::Next => row((text("Next"), icon("chevron-right"))).into_view(),
            PageItem::Page(n) => text((n + 1).to_string()).into_view(),
            _ => icon("ellipsis").into_view(),
        })
}
// end example

// example: labels
fn labels() -> impl IntoView {
    let page = signal(1usize);
    kivact::ui::pagination(5usize, page).label_format(|number| format!("p.{number}"))
}
// end example

// example: long
fn long() -> impl IntoView {
    let page = signal(41usize);
    kivact::ui::pagination(100usize, page).item(|item| match item {
        PageItem::Previous => row((icon("chevron-left"), text("Previous"))).into_view(),
        PageItem::Next => row((text("Next"), icon("chevron-right"))).into_view(),
        PageItem::Page(n) => text((n + 1).to_string()).into_view(),
        _ => icon("ellipsis").into_view(),
    })
}
// end example
