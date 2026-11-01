//! Role: Catalog page of Breadcrumb: basic trail, ellipsis and right-to-left.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "breadcrumb",
        name: "Breadcrumb",
        description: "現在のページまでの階層を示す部品です。「Breadcrumb」という名前の Navigation として公開され、direction の中では向きが反転します。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("breadcrumb.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "breadcrumb_link、breadcrumb_separator、breadcrumb_page を並べます。リンクは on_press で押せます。",
                default,
            ),
            Example::new(
                "ellipsis",
                "Ellipsis",
                "breadcrumb_ellipsis で省略した階層を表します。",
                ellipsis,
            ),
            Example::new(
                "rtl",
                "Right to left",
                "basic::direction で Rtl を渡すと、並びと区切りの向きが反転します。",
                rtl,
            ),
        ],
    }
}

// example: default
fn default() -> impl IntoView {
    let at = signal(String::from("Components"));
    basic::breadcrumb_content((
        basic::breadcrumb_link("Home").on_press(move || at.set("Home".into())),
        basic::breadcrumb_separator(),
        basic::breadcrumb_link("Components").on_press(move || at.set("Components".into())),
        basic::breadcrumb_separator(),
        basic::breadcrumb_page(at),
    ))
}
// end example

// example: ellipsis
fn ellipsis() -> impl IntoView {
    basic::breadcrumb_content((
        basic::breadcrumb_link("Home").on_press(|| {}),
        basic::breadcrumb_separator(),
        basic::breadcrumb_ellipsis(),
        basic::breadcrumb_separator(),
        basic::breadcrumb_link("Components").on_press(|| {}),
        basic::breadcrumb_separator(),
        basic::breadcrumb_page("Breadcrumb"),
    ))
}
// end example

// example: rtl
fn rtl() -> impl IntoView {
    kivact::view::provide_direction(kivact::view::ReadingDirection::Rtl, || {
        basic::breadcrumb_content((
            basic::breadcrumb_link("Home").on_press(|| {}),
            basic::breadcrumb_separator(),
            basic::breadcrumb_link("Docs").on_press(|| {}),
            basic::breadcrumb_separator(),
            basic::breadcrumb_page("Page"),
        ))
        .style(style().width(280))
    })
}
// end example
